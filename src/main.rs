//#![allow(warnings)]
use std::fs;
use std::time::Instant;
use std::fmt::Write;

use itertools::Itertools;

use screeps::local::{LocalRoomTerrain, RoomName};

use screeps_utils::offline_map;

use screeps_room_regions;

use image;
use image::{GenericImageView, GenericImage, RgbaImage, Rgba};
use render::OutputImage;
use screeps_local_visuals::render;

mod local_display_utils;

const OUTPUT_DIR: &str = "./output_images";

/// Build a simple html file to show all of the
/// images with before and after.
fn make_image_html_index(room_names: &[&str]) {
  let mut html_image_list = String::new();

  for room_name in room_names {
    write!(&mut html_image_list, r#"
  <h3>{room_name}</h3>
  <div class="image-row">
    <img class="image" src="./{room_name}_0.png" />
    <img class="image" src="./{room_name}_1.png" />
  </div>"#)
      .expect("Write to string failed");
  }
  let styles_body = r##"
    .image-column {
      display: flex;
      flex-flow: column nowrap;
    }

    .image-row {
      width: 80%;
      display: flex;
      flex-flow: row nowrap;
    }

    .image {
      flex: 50%;
      width: 50%;
    }
    "##;
  fs::write(format!("{OUTPUT_DIR}/index.html"), format!(r##"
<!DOCTYPE html>
<html lang="en">

<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <meta http-equiv="Cache-Control" content="no-cache, no-store, must-revalidate" />
  <meta http-equiv="Pragma" content="no-cache" />
  <meta http-equiv="Expires" content="0" />
  <title>View for Images</title>
  <style>{styles_body}</style>
</head>

<body>
  <h1>View Images</h1>
  <div class="image-column">
    {html_image_list}
  </div>
</body>

</html>

"##)).expect("Couldn't write to index file");
}

fn main() {
  fs::create_dir_all(OUTPUT_DIR).expect("Couldn't make output directory");

  let path = std::path::Path::new("./test_data/map-mmo-shard3.json");

  let shard_data = offline_map::load_shard_map_json(path);

  // let rooms_to_plan = vec!("W58N23");
  let rooms_to_plan = vec!["W56N22", "W49N48", "W49N46", "W48N46"];
  make_image_html_index(&rooms_to_plan);

  for room_name in rooms_to_plan {
    if let Ok(rn) = RoomName::new(room_name) {
      if let Some(room_data) = shard_data.rooms.get(&rn) {
        println!("Room: {:?}", room_name);
        let output = make_images_for_room(room_data);
        for (i, imgbuf) in output.iter().enumerate() {
          let filename = format!("{}/{}_{}.png", OUTPUT_DIR, room_name, i);
          imgbuf.save(filename).unwrap();
        }
      }
    }
  }
}

fn make_images_for_room(room_data: &offline_map::OfflineRoomData) -> Vec<OutputImage> {
  let mut output = Vec::new();

  let mut base_imgbuf = render::create_image();

  local_display_utils::draw_terrain_from_offline_terrain_data(&mut base_imgbuf, &room_data.terrain);
  local_display_utils::draw_resources_from_offline_room_data(&mut base_imgbuf, &room_data);
  local_display_utils::draw_structures_from_offline_room_data(&mut base_imgbuf, &room_data);

  output.push(base_imgbuf.clone());

  let mut imgbuf = base_imgbuf.clone();
  test_region_analysis(&mut imgbuf, &room_data.terrain);
  output.push(imgbuf);

  output
}

fn test_region_analysis(imgbuf: &mut OutputImage, terrain: &LocalRoomTerrain) {
  let now = Instant::now();

  let region_analysis = screeps_room_regions::get_region_analysis_for_room_by_terrain(terrain);

  println!("Time to Analyze: {:?}", now.elapsed());

  let now = Instant::now();

  let regions = region_analysis.get_regions();
  let num_regions_raw: u8 = regions.len().try_into().unwrap();
  let num_regions: u8 = num_regions_raw + 1; // Plus 1 for the borders we color later
  for (idx, region) in regions.iter().enumerate() {
    let region_color_rbg = calculate_color(num_regions, idx.try_into().unwrap());
    let region_members = region.get_members();

    let region_tiles: Vec<_> = region_members
      .iter()
      .map(|xy| (xy.x.u8(), xy.y.u8()))
      .collect();
    let alpha_overlay = render::get_tile_alpha_overlay_multi_tile(
      imgbuf.width(),
      imgbuf.height(),
      render::DEFAULT_SCALE_FACTOR,
      region_color_rbg.0,
      region_color_rbg.1,
      region_color_rbg.2,
      128,
      &region_tiles,
    );
    image::imageops::overlay(imgbuf, &alpha_overlay, 0, 0);

    for xy in region_members {
      let height = region_analysis.get_height_for_xy(&xy);
      render::draw_text_number_xy(
        imgbuf,
        xy.x.u8().into(),
        xy.y.u8().into(),
        &height.to_string(),
      );
    }
  }

  // let mut i: u8 = 0;
  // for (color_a, color_b, border_segment) in region_analysis.get_border_segments().iter() {
  //   i += 1;
  //   for xy in border_segment.walls() {
  //     // let height = region_analysis.get_height_for_xy(&xy);
  //     render::draw_text_number_xy(imgbuf, xy.x.u8().into(), xy.y.u8().into(), &i.to_string());
  //   }
  // }

  let border_tiles: Vec<_> = region_analysis
    .get_border_tiles()
    .iter()
    .map(|xy| (xy.x.u8(), xy.y.u8()))
    .collect();
  for (x, y) in border_tiles {
    render::draw_text_number_xy(imgbuf, x.into(), y.into(), "B");
  }

  println!("Time to Render: {:?}", now.elapsed());
}

/// Generate a nice color for our visuals.
fn calculate_color(size: u8, index: u8) -> (u8, u8, u8) {
  let percentage = f64::from(index) / f64::from(size);
  let radians: f64 = percentage * core::f64::consts::TAU;
  // see: https://stackoverflow.com/questions/10731147/evenly-distributed-color-range-depending-on-a-count
  // we basically pick a spot on the color wheel and then turn
  // that into the U/V plane with fixed brightness.
  let u: f64 = radians.cos();
  let v: f64 = radians.sin();
  let y: f64 = 1.0; // brightness.

  // now we convert into RGB.
  let red: f64 = y + v / 0.88;
  let green: f64 = y - 0.38 * u - 0.58 * v;
  let blue: f64 = y + u / 0.49;

  /// Convert one of the generated RGB values into an actual
  /// byte value.
  fn convert(val: f64) -> u8 {
    let rgb_percent = val.clamp(0.0, 2.0) / 2.0;
    let rgb_float = rgb_percent * 255.0;
    rgb_float.floor() as u8
  }

  // now we turn our floats into bytes
  // we have to clamp because the YUV color space is bizzare.
  let redb: u8 = convert(red);
  let greenb: u8 = convert(green);
  let blueb: u8 = convert(blue);

  (redb, greenb, blueb)
}
//*/
