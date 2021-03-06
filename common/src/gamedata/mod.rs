
pub mod meta;
pub mod chara;
pub mod defs;
pub mod item;
pub mod map;
pub mod player;
pub mod quest;
pub mod region;
pub mod shop;
pub mod site;
pub mod skill;
pub mod time;
pub mod town;
pub mod variables;

use array2d::Vec2d;

pub use self::meta::*;
pub use self::chara::*;
pub use self::defs::*;
pub use self::item::*;
pub use self::map::*;
pub use self::player::*;
pub use self::quest::*;
pub use self::region::*;
pub use self::shop::*;
pub use self::site::*;
pub use self::skill::*;
pub use self::time::*;
pub use self::town::*;
pub use self::variables::*;

/// Includes all data for one game world.
/// This can be a snapshot of the current game, so it must implement Serialize and Deserialize.
#[derive(Serialize, Deserialize)]
pub struct GameData {
    #[serde(skip)]
    pub meta: MetaData,
    pub chara: CharaHolder,
    pub region: RegionHolder,
    pub time: Time,
    pub player: Player,
    pub quest: QuestHolder,
    pub vars: Variables,
    current_mapid: MapId,
}

impl GameData {
    pub fn empty() -> GameData {
        GameData {
            meta: MetaData::default(),
            chara: CharaHolder::new(),
            region: RegionHolder::new(),
            time: Time::default(),
            player: Player::default(),
            quest: QuestHolder::new(),
            vars: Variables::new(),
            current_mapid: MapId::default(),
        }
    }

    pub fn get_current_mapid(&self) -> MapId {
        self.current_mapid
    }

    pub fn get_current_map(&self) -> &Map {
        self.region.get_map(self.current_mapid)
    }

    pub fn get_current_map_mut(&mut self) -> &mut Map {
        self.region.get_map_mut(self.current_mapid)
    }

    pub fn get_current_region_id(&self) -> RegionId {
        let mapid = self.get_current_mapid();
        mapid.rid()
    }

    pub fn get_current_region(&self) -> &Region {
        self.region.get(self.current_mapid.rid())
    }

    pub fn get_current_region_mut(&mut self) -> &mut Region {
        self.region.get_mut(self.current_mapid.rid())
    }

    pub fn add_chara(&mut self, chara: Chara, kind: CharaKind) -> CharaId {
        match kind {
            CharaKind::Player => {
                self.chara.add(CharaId::Player, chara);
                CharaId::Player
            }
            CharaKind::OnSite => {
                panic!("Adding OnSite chara without id is unavailable")
            }
            CharaKind::OnMap => {
                panic!("Adding OnMap chara without mapid is unavailable")
            }
        }
    }

    /// Add chara as OnMap
    pub fn add_chara_to_map(&mut self, chara: Chara, mid: MapId) -> CharaId {
        let cid = CharaId::OnMap { mid, n: self.region.get_map(mid).search_empty_onmap_charaid_n() };

        if mid == self.current_mapid {
            self.chara.add(cid, chara);
        } else {
            let map = self.region.get_map_mut(mid);
            map.charas.as_mut().unwrap().insert(cid, chara);
        }
        cid
    }

    /// Add chara as OnSite
    pub fn add_chara_to_site(&mut self, chara: Chara, sid: SiteId, n: u32) -> CharaId {
        let cid = CharaId::OnSite { sid, n };
        self.chara.add(cid, chara);
        cid
    }

    /// Remove specified character from game.
    /// If the character is in the current map, remove from map data
    pub fn remove_chara(&mut self, cid: CharaId) {
        match cid {
            CharaId::Player => {
                panic!();
            }
            CharaId::OnMap { mid, .. } => {
                let map = self.region.get_map_mut(mid);
                map.remove_chara(cid);
            }
            _ => {
                self.get_current_map_mut().remove_chara(cid);
            },
        }
        self.chara.remove_chara(cid);
    }

    pub fn add_site(&mut self, site: Site, kind: SiteKind, rid: RegionId, pos: Vec2d) -> Option<SiteId> {
        let region = self.region.get_mut(rid);
        region.add_site(site, kind, pos)
    }

    pub fn add_map(&mut self, map: Map, sid: SiteId, map_random_id: u64) -> MapId {
        let site = self.region.get_site_mut(sid);
        let floor = site.add_map(map, map_random_id);
        MapId::SiteMap { sid, floor }
    }

    pub fn set_current_mapid(&mut self, mid: MapId) {
        // OnMap characters on the next map
        let next_charas = self.region.get_map_mut(mid).charas.take().expect("Map.charas is empty");
        let prev_charas = self.chara.replace_on_map_chara(next_charas);
        let map = self.get_current_map_mut();
        assert!(map.charas.is_none());
        map.charas = Some(prev_charas);

        // Update current_mapid
        self.current_mapid = mid;
    }

    pub fn set_initial_mapid(&mut self, mid: MapId) {
        let charas = self.region.get_map_mut(mid).charas.take().unwrap();
        self.chara.replace_on_map_chara(charas);
        self.current_mapid = mid;
    }

    // Fuctions for item handling

    /// Get item list by ItemListLocation
    pub fn get_item_list(&self, list_location: ItemListLocation) -> &ItemList {
        match list_location {
            ItemListLocation::Chara { cid } => {
                &self.chara.get(cid).item_list
            }
            ItemListLocation::Equip { cid } => {
                self.chara.get(cid).equip.list()
            }
            ItemListLocation::OnMap { mid, pos } => {
                &self.region.get_map(mid).tile[pos].item_list.as_ref().expect("Get item list to empty tile")
            }
            ItemListLocation::Shop { cid } => {
                &self.get_shop(cid).items
            }
        }
    }

    /// Mutable version for get_item_list
    pub fn get_item_list_mut(&mut self, list_location: ItemListLocation) -> &mut ItemList {
        match list_location {
            ItemListLocation::Chara { cid } => {
                &mut self.chara.get_mut(cid).item_list
            }
            ItemListLocation::Equip { .. } => {
                panic!("Mutable borrow is prohibited for equipment list");
            }
            ItemListLocation::OnMap { mid, pos } => {
                self.region.get_map_mut(mid).tile[pos].item_list.as_mut()
                    .expect("Get item list to empty tile")
            }
            ItemListLocation::Shop { cid } => {
                &mut self.get_shop_mut(cid).items
            }
        }
    }

    pub fn get_item(&self, item_location: ItemLocation) -> (&Item, u32) {
        let a = &self.get_item_list(item_location.0).items[item_location.1 as usize];
        (&a.0, a.1)
    }

    /// Remove item from list
    pub fn remove_item<T: Into<ItemMoveNum>>(&mut self, item_location: ItemLocation, n: T) {
        {
            let item_list = self.get_item_list_mut(item_location.0);
            item_list.remove(item_location.1, n);
        }
        self.check_item_list_on_tile(item_location.0);
    }

    /// Remove item from list and get its clone or moved value
    pub fn remove_item_and_get<T: Into<ItemMoveNum>>(&mut self, item_location: ItemLocation, n: T)
                                             -> Item {
        let result = {
            let item_list = self.get_item_list_mut(item_location.0);
            item_list.remove_and_get(item_location.1, n)
        };
        self.check_item_list_on_tile(item_location.0);
        result
    }

    /// Move item to dest
    pub fn move_item<T: Into<ItemMoveNum>>(&mut self, item_location: ItemLocation,
                                           dest: ItemListLocation, n: T) {
        let (item, n) = {
            let src_list = self.get_item_list_mut(item_location.0);
            let n = match n.into() {
                ItemMoveNum::Partial(n) => n,
                ItemMoveNum::All => {
                    src_list.get_number(item_location.1)
                }
            };
            (src_list.remove_and_get(item_location.1, n), n)
        };
        
        self.create_item_list_on_tile(dest);
        let dest_list = self.get_item_list_mut(dest);
        dest_list.append(item, n);
        
        self.check_item_list_on_tile(item_location.0);
    }

    /// Checks item list on tile is empty or not. If so, delete
    fn check_item_list_on_tile(&mut self, item_list_location: ItemListLocation) {
        match item_list_location {
            ItemListLocation::OnMap { mid, pos } => {
                if self.get_item_list(item_list_location).is_empty() {
                    self.region.get_map_mut(mid).tile[pos].item_list = None;
                }
            }
            _ => (),
        }
    }

    /// Create item list on tile if empty
    fn create_item_list_on_tile(&mut self, item_list_location: ItemListLocation) {
        match item_list_location {
            ItemListLocation::OnMap { mid, pos } => {
                if self.region.get_map_mut(mid).tile[pos].item_list.is_none() {
                    self.region.get_map_mut(mid).tile[pos].item_list
                        = Some(ItemList::new());
                }
            }
            _ => (),
        }
    }

    pub fn get_equip_list(&self, cid: CharaId) -> &EquipItemList {
        let chara = self.chara.get(cid);
        &chara.equip
    }

    pub fn get_equip_list_mut(&mut self, cid: CharaId) -> &mut EquipItemList {
        let chara = self.chara.get_mut(cid);
        &mut chara.equip
    }

    pub fn get_shop(&self, cid: CharaId) -> &Shop {
        let (sid, n) = match cid {
            CharaId::OnSite { sid, n } => (sid, n),
            _ => panic!("Tried to get shop according to no OnSite character"),
        };
        let site = self.region.get_site(sid);
        match &site.content {
            SiteContent::Town { ref town } => {
                town.get_shop(n)
                    .unwrap_or_else(|| panic!("Shop #{} doesnot exit in {:?}", n, sid))
            }
            _ => unreachable!("Tried to get shop in a site that is not town"),
        }
    }

    pub fn get_shop_mut(&mut self, cid: CharaId) -> &mut Shop {
        let (sid, n) = match cid {
            CharaId::OnSite { sid, n } => (sid, n),
            _ => panic!("Tried to get shop according to no OnSite character"),
        };
        let site = self.region.get_site_mut(sid);
        match &mut site.content {
            SiteContent::Town { ref mut town } => {
                town.get_shop_mut(n)
                    .unwrap_or_else(|| panic!("Shop #{} doesnot exit in {:?}", n, sid))
            }
            _ => unreachable!("Tried to get shop in a site that is not town"),
        }
    }
}

fn unknown_id_err<T: std::fmt::Debug>(id: T) -> ! {
    panic!("Internal error: Unknown id - {:?}", id)
}

