use quicksilver::{
    geom::{Rectangle, Scalar, Shape, Vector},
    graphics::{Color, Graphics, Image},
    lifecycle::event::KeyboardEvent,
    lifecycle::{run, Event, EventStream, Key, Settings, Window},
    Result,
};
use space_invaders::utils::timer::Timer;
use std::borrow::Borrow;
use std::collections::HashMap;

fn main() {
    run(
        Settings {
            size: Vector::new(800.0, 600.0).into(),
            title: "Space invaders",
            ..Settings::default()
        },
        app,
    );
}

async fn app(window: Window, mut gfx: Graphics, mut events: EventStream) -> Result<()> {
    // Clear the screen to a blank, white color
    gfx.clear(Color::WHITE);

    let mut game = Game::new();
    game.init(&mut gfx).await;

    let mut update_timer = Timer::time_per_second(30.0);
    let mut draw_timer = Timer::time_per_second(60.0);

    loop {
        while let Some(event) = events.next_event().await {
            match event {
                Event::KeyboardInput(event) => game.handle_key(event),
                _ => (),
            }
        }

        // We use a while loop rather than an if so that we can try to catch up in the event of having a slow down.
        while update_timer.tick() {
            game.update()
        }

        if draw_timer.exhaust().is_some() {
            gfx.clear(Color::BLACK);

            game.render(&mut gfx);
            gfx.present(&window)?;
        }
    }
}

#[derive(Eq, PartialEq)]
enum Movement {
    None,
    Left,
    Right,
}

pub struct Game {
    player: Entity,
    enemies: Vec<Entity>,
    bullets: Vec<Entity>,
    player_movement: Movement,
    images: HashMap<String, Box<Image>>,
}

impl Game {
    pub fn new() -> Self {
        let images: HashMap<String, Box<Image>> = HashMap::new();

        Self {
            player: Entity::new_player(400.0, 550.0, EntityView::None),
            enemies: vec![],
            bullets: vec![],
            player_movement: Movement::None,
            images,
        }
    }
    pub async fn init(&mut self, gfx: &mut Graphics) {
        self.images.insert(
            String::from("player"),
            Box::new(Image::load(&gfx, "player.png").await.unwrap()),
        );
        self.images.insert(
            String::from("enemy"),
            Box::new(Image::load(&gfx, "enemy.png").await.unwrap()),
        );

        self.player.set_view(EntityView::Image(
            self.images.get("player").unwrap().clone(),
        ));
        for i in 1..10 {
            self.enemies.push(Entity::new_enemy(
                150.0 + i.float() * 50.0,
                20.0,
                EntityView::Image(self.images.get("enemy").unwrap().clone()),
            ))
        }
    }

    pub fn update(&mut self) {
        match self.player_movement {
            Movement::Left => self.player.move_left(10.0),
            Movement::Right => self.player.move_right(10.0),
            _ => (),
        }

        for enemy in &mut self.enemies.iter_mut() {
            enemy.move_down(1.0);
        }

        for bullet in &mut self.bullets.iter_mut() {
            bullet.move_up(20.0);

            self.enemies.retain(|enemy| !bullet.hit_test(enemy))
        }
    }

    pub fn handle_key(&mut self, event: KeyboardEvent) {
        match event.key() {
            Key::Left => {
                if event.is_down() {
                    self.player_movement = Movement::Left
                } else if self.player_movement == Movement::Left {
                    self.player_movement = Movement::None
                }
            }
            Key::Right => {
                if event.is_down() {
                    self.player_movement = Movement::Right
                } else if self.player_movement == Movement::Right {
                    self.player_movement = Movement::None
                }
            }
            Key::Space => {
                let Vector { x, y } = self.player.center();
                self.bullets
                    .push(Entity::new_bullet(x, y, EntityView::Color(Color::GREEN)))
            }
            _ => (),
        }
    }
}

impl Renderable for Game {
    fn render(&self, gfx: &mut Graphics) {
        self.player.render(gfx);
        for enemy in self.enemies.iter() {
            enemy.render(gfx)
        }
        for bullet in self.bullets.iter() {
            bullet.render(gfx)
        }
    }
}

#[derive(Clone)]
pub enum EntityView {
    None,
    Image(Box<Image>),
    Color(Color),
}

pub struct Entity {
    rect: Rectangle,
    view: EntityView,
}

trait Renderable {
    fn render(&self, gfx: &mut Graphics);
}

impl Entity {
    pub fn new_player(x: f32, y: f32, view: EntityView) -> Self {
        Self::new(x, y, 30.0, 30.0, view)
    }
    pub fn new_enemy(x: f32, y: f32, view: EntityView) -> Self {
        Self::new(x, y, 30.0, 30.0, view)
    }
    pub fn new_bullet(x: f32, y: f32, view: EntityView) -> Self {
        Self::new(x, y, 2.0, 30.0, view)
    }

    pub fn new(x: f32, y: f32, width: f32, height: f32, view: EntityView) -> Self {
        Self {
            rect: Rectangle::new(
                Vector::new(x - width / 2.0, y - height / 2.0),
                Vector::new(width, height),
            ),
            view,
        }
    }

    pub fn hit_test(&self, entity: &Entity) -> bool {
        self.rect.overlaps_rectangle(&entity.rect)
    }

    pub fn move_left(&mut self, amount: f32) {
        self.rect.pos.x -= amount
    }

    pub fn move_right(&mut self, amount: f32) {
        self.rect.pos.x += amount
    }

    pub fn move_down(&mut self, amount: f32) {
        self.rect.pos.y += amount
    }

    pub fn move_up(&mut self, amount: f32) {
        self.rect.pos.y -= amount
    }

    pub fn center(&self) -> Vector {
        let Rectangle {
            pos: Vector { x, y },
            size: Vector { x: w, y: h },
        } = self.rect;
        Vector::new(x + w / 2.0, y + h / 2.0)
    }

    pub fn set_view(&mut self, view: EntityView) {
        self.view = view
    }
}

impl Renderable for Entity {
    fn render(&self, gfx: &mut Graphics) {
        match self.view.clone() {
            EntityView::None => (),
            EntityView::Image(image) => {
                let region = Rectangle::new(self.rect.pos, self.rect.size());
                gfx.draw_image(image.borrow(), region);
            }
            EntityView::Color(color) => gfx.fill_rect(&self.rect, color),
        }
    }
}
