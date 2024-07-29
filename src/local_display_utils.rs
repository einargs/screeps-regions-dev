use image;
use image::{GenericImage, GenericImageView};

use screeps::constants::extra::ROOM_SIZE;

use screeps::RoomXY;

use screeps_utils::offline_map;

use render::{OutputImage, DEFAULT_SCALE_FACTOR};
use screeps_local_visuals::render;

//use screeps_clockwork_army::base_planning;
//use base_planning::structure_placement_structs::BuildPlan;

pub fn render_roomxy_path(imgbuf: &mut OutputImage, path: &[RoomXY]) {
  let start_rgba = (255, 0, 0, 125);
  let end_rgba = (0, 255, 0, 125);
  let path_rgba = (0, 0, 255, 125);

  let start_index = 0;
  let end_index = path.len() - 1;

  let start_xy = path[start_index];
  let end_xy = path[end_index];
  let middle_xy_slice = &path[1..end_index];

  let scale_factor = DEFAULT_SCALE_FACTOR;

  let alpha_overlay = render::get_tile_alpha_overlay(
    imgbuf.width(),
    imgbuf.height(),
    scale_factor,
    start_rgba.0,
    start_rgba.1,
    start_rgba.2,
    start_rgba.3,
    start_xy.x.u8(),
    start_xy.y.u8(),
  );
  image::imageops::overlay(imgbuf, &alpha_overlay, 0, 0);

  let middle_tiles: Vec<_> = middle_xy_slice
    .into_iter()
    .map(|xy| (xy.x.u8(), xy.y.u8()))
    .collect();
  let alpha_overlay = render::get_tile_alpha_overlay_multi_tile(
    imgbuf.width(),
    imgbuf.height(),
    scale_factor,
    path_rgba.0,
    path_rgba.1,
    path_rgba.2,
    path_rgba.3,
    &middle_tiles,
  );
  image::imageops::overlay(imgbuf, &alpha_overlay, 0, 0);

  let alpha_overlay = render::get_tile_alpha_overlay(
    imgbuf.width(),
    imgbuf.height(),
    scale_factor,
    end_rgba.0,
    end_rgba.1,
    end_rgba.2,
    end_rgba.3,
    end_xy.x.u8(),
    end_xy.y.u8(),
  );
  image::imageops::overlay(imgbuf, &alpha_overlay, 0, 0);
}

/*
pub fn render_buildplan(imgbuf: &mut OutputImage, build_plan: &BuildPlan) {
  for structure_placement in build_plan.get_all_structure_placements() {
    let x = structure_placement.pos.x;
    let y = structure_placement.pos.y;
    render::draw_buildablestructure_tile_xy(
      imgbuf,
      x.u8().into(),
      y.u8().into(),
      &structure_placement.structure_type.try_into().unwrap(),
    );
  }
}

pub fn render_buildplan_at_rcl(imgbuf: &mut OutputImage, build_plan: &BuildPlan, rcl: u8) {
  for structure_placement in build_plan.get_all_structure_placements() {
    if structure_placement.rcl <= rcl {
      let x = structure_placement.pos.x;
      let y = structure_placement.pos.y;
      render::draw_buildablestructure_tile_xy(
        imgbuf,
        x.u8().into(),
        y.u8().into(),
        &structure_placement.structure_type.try_into().unwrap(),
      );
    }
  }
}
*/

pub fn draw_terrain_from_offline_terrain_data(
  imgbuf: &mut OutputImage,
  terrain_data: &screeps::local::LocalRoomTerrain,
) {
  for col in 0..ROOM_SIZE {
    for row in 0..ROOM_SIZE {
      if let Ok(position) = screeps::local::RoomXY::try_from((col as u8, row as u8)) {
        render::draw_terrain_tile_xy(
          imgbuf,
          col.into(),
          row.into(),
          &terrain_data.get_xy(position),
        )
      }
    }
  }
}

pub fn draw_resources_from_offline_room_data(
  imgbuf: &mut OutputImage,
  room_data: &offline_map::OfflineRoomData,
) {
  for obj in &room_data.objects {
    match obj {
      offline_map::OfflineObject::Source { x, y, .. } => render::draw_resource_tile_xy(
        imgbuf,
        x.u8().into(),
        y.u8().into(),
        &obj.try_into().unwrap(),
      ),
      offline_map::OfflineObject::Mineral {
        x, y, mineral_type, ..
      } => render::draw_resource_tile_xy(
        imgbuf,
        x.u8().into(),
        y.u8().into(),
        &mineral_type.try_into().unwrap(),
      ),
      _ => (),
    }
  }
}

pub fn draw_structures_from_offline_room_data(
  imgbuf: &mut OutputImage,
  room_data: &offline_map::OfflineRoomData,
) {
  for obj in &room_data.objects {
    use offline_map::OfflineObject::*;
    match obj {
      ConstructedWall { x, y, .. } => render::draw_buildablestructure_tile_xy(
        imgbuf,
        x.u8().into(),
        y.u8().into(),
        &obj.try_into().unwrap(),
      ),
      Controller { x, y, .. } => render::draw_buildablestructure_tile_xy(
        imgbuf,
        x.u8().into(),
        y.u8().into(),
        &obj.try_into().unwrap(),
      ),
      Extractor { x, y, .. } => render::draw_buildablestructure_tile_xy(
        imgbuf,
        x.u8().into(),
        y.u8().into(),
        &obj.try_into().unwrap(),
      ),
      Terminal { x, y, .. } => render::draw_buildablestructure_tile_xy(
        imgbuf,
        x.u8().into(),
        y.u8().into(),
        &obj.try_into().unwrap(),
      ),
      _ => (),
    }
  }
}
