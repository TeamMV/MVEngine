use hashbrown::HashMap;

type Key = String;

pub struct Language {
    pub name: String,
    pub region: String,
    pub code: String,
    left_to_right: bool,
    lookup: HashMap<Key, String>,
}

impl Language {
    // pub fn parse(inp: &'static str) -> Self {
    //     let lines = inp.lines();
    //     let mut map = HashMap::new();
    //
    //
    //     //infrsasturucoter
    //     /*
    //     Game struct thingy
    //      > ecs
    //      > langauage
    //      > io
    //      > ...
    //
    //     let mygame = Game::new(game_info);
    //     let window = Window::new(win_info);
    //     window.run::<AppLoop>(game);
    //     --> post_init(window: &mut Window, game: &mut Game)
    //
    //     game.events.dispatch::<MyGameEvent>(IdkEvent::new());
    //
    //      */
    //
    //     for line in lines {
    //         if let Some((key, value)) = line.split_once('=') {
    //             map.insert(key.to_string(), value.to_string());
    //         }
    //     }
    //     Ok()
    // }

    pub fn lookup(&self, key: &str) -> Option<&String> {
        self.lookup.get(key)
    }

    pub fn has(&self, key: &str) -> bool {
        self.lookup.contains_key(key)
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn region(&self) -> &str {
        &self.region
    }

    pub fn code(&self) -> &str {
        &self.code
    }

    pub fn left_to_right(&self) -> bool {
        self.left_to_right
    }
}
