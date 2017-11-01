
use common::gamedata;
use common::gamedata::GameData;
use super::site::add_dungeon_site;
use super::map;

pub fn create_newgame() -> GameData {
    let mut gd = GameData::empty();
    
    add_dungeon_site(&mut gd);

    let mid = gd.get_current_mapid();
    let start_pos = gd.get_current_map().entrance;

    let mut chara = gamedata::chara::Chara::default();
    chara.params.spd = 100;
    chara.params.str = 25;
    chara.rel = gamedata::chara::Relationship::ALLY;
    gd.add_chara_to_map(chara, gamedata::chara::CharaKind::Player, mid, start_pos);

    map::gen_npcs(&mut gd, mid, 10, 10);

    gd
}

