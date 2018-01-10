use std::rc::Rc;
use std::cell::RefCell;
use std::cmp;

use grt::ui::{Cursor, Label, WidgetKind, Widget};
use grt::io::*;
use grt::io::event::ClickKind;
use grt::util::Point;
use grt::resource::ResourceSet;

use extern_image::{ImageBuffer, Rgba};

use view::ActionMenu;
use state::GameState;

pub struct AreaView {
    mouse_over: Rc<RefCell<Widget>>,
    scale: RefCell<(f32, f32)>,
    cursors: RefCell<Option<DrawList>>,
    buffer: RefCell<ImageBuffer<Rgba<u8>, Vec<u8>>>,
    cache_invalid: RefCell<bool>,
    current_area_id: RefCell<String>,
}

const TILE_CACHE_TEXTURE_SIZE: u32 = 1024;

impl AreaView {
    pub fn new(mouse_over: Rc<RefCell<Widget>>) -> Rc<AreaView> {
        Rc::new(AreaView {
            mouse_over: mouse_over,
            scale: RefCell::new((1.0, 1.0)),
            cursors: RefCell::new(None),
            buffer: RefCell::new(ImageBuffer::new(0, 0)),
            cache_invalid: RefCell::new(true),
            current_area_id: RefCell::new(String::new()),
        })
    }

    pub fn clear_cursors(&self) {
        *self.cursors.borrow_mut() = None;
    }

    pub fn add_cursor(&self, mut cursor: DrawList) {
        match *self.cursors.borrow_mut() {
            Some(ref mut c) => {
                c.append(&mut cursor);
                return;
            },
            None => {},
        };

        *self.cursors.borrow_mut() = Some(cursor);
    }

    fn get_cursor_pos_no_scroll(&self, widget: &Rc<RefCell<Widget>>) -> (i32, i32) {
        self.get_cursor_pos_scaled(widget.borrow().state.position.x,
            widget.borrow().state.position.y)
    }

    fn get_cursor_pos(&self, widget: &Rc<RefCell<Widget>>) -> (i32, i32) {
        let pos = widget.borrow().state.position;
        self.get_cursor_pos_scaled(pos.x - widget.borrow().state.scroll_pos.x
                                   , pos.y - widget.borrow().state.scroll_pos.y)
    }

    fn get_cursor_pos_scaled(&self, pos_x: i32, pos_y: i32) -> (i32, i32) {
        let mut x = Cursor::get_x_f32() - pos_x as f32;
        let mut y = Cursor::get_y_f32() - pos_y as f32;

        let (scale_x, scale_y) = *self.scale.borrow();
        x = x / scale_x;
        y = y / scale_y;

        (x as i32, y as i32)
    }

    fn draw_tiles_to_buffer(&self, buffer: &mut ImageBuffer<Rgba<u8>, Vec<u8>>) {
        let area_state = GameState::area_state();
        let state = area_state.borrow();
        let ref area = state.area;
        let max_tile_x = cmp::min(buffer.width() as i32 / 16, area.width);
        let max_tile_y = cmp::min(buffer.height() as i32 / 16, area.height);
        let spritesheet = ResourceSet::get_spritesheet("tiles").unwrap();
        let source = &spritesheet.image;
        trace!("Generating cached tiles for '{}'", area.id);

        for tile_y in 0..max_tile_y {
            for tile_x in 0..max_tile_x {
                let dest_x = 16 * tile_x as u32;
                let dest_y = 16 * tile_y as u32;

                let tile = match area.terrain.tile_at(tile_x, tile_y) {
                    &None => continue,
                    &Some(ref tile) => tile,
                };

                let sprite = &tile.image_display;
                let src_x = sprite.position.x as u32;
                let src_y = sprite.position.y as u32;
                for y in 0..sprite.size.height {
                    for x in 0..sprite.size.width {
                        buffer.put_pixel(dest_x + x as u32, dest_y + y as u32,
                                         *source.get_pixel(src_x + x as u32, src_y + y as u32));
                    }
                }
            }
        }
    }
}

impl WidgetKind for AreaView {
    fn get_name(&self) -> &str {
        "area"
    }

    fn on_add(&self, widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        self.clear_cursors();
        let area_state = GameState::area_state();

        let width = area_state.borrow().area.width;
        let height = area_state.borrow().area.height;
        widget.borrow_mut().state.set_max_scroll_pos(width, height);
        self.mouse_over.borrow_mut().state.add_text_param("");
        self.mouse_over.borrow_mut().state.add_text_param("");

        let mut buffer: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(TILE_CACHE_TEXTURE_SIZE,
                                                                          TILE_CACHE_TEXTURE_SIZE);
        self.draw_tiles_to_buffer(&mut buffer);
        *self.buffer.borrow_mut() = buffer;
        *self.cache_invalid.borrow_mut() = true;

        Vec::with_capacity(0)
    }

    fn draw_text_mode(&self, renderer: &mut TextRenderer,
                      widget: &Widget, _millis: u32) {
        let p = widget.state.inner_position;
        let s = widget.state.inner_size;
        let scroll = widget.state.scroll_pos;

        let area_state = GameState::area_state();
        let ref area = area_state.borrow().area;

        let max_x = cmp::min(s.width, area.width - scroll.x);
        let max_y = cmp::min(s.height, area.height - scroll.y);

        renderer.set_cursor_pos(0, 0);

        for y in 0..max_y {
            renderer.set_cursor_pos(p.x, p.y + y);
            for x in 0..max_x {
                renderer.render_char(area_state.borrow().get_display(x + scroll.x,
                                                                     y + scroll.y));
            }
        }
    }

    fn draw_graphics_mode(&self, renderer: &mut GraphicsRenderer, pixel_size: Point,
                          widget: &Widget, _millis: u32) {
        let scale_x = 1600.0 / (pixel_size.x as f32);
        let scale_y = 900.0 / (pixel_size.y as f32);
        *self.scale.borrow_mut() = (scale_x, scale_y);

        let p = widget.state.inner_position;
        let inner_width = (widget.state.inner_size.width as f32 / scale_x).round() as i32;
        let inner_height = (widget.state.inner_size.height as f32 / scale_y).round() as i32;

        let area_state = GameState::area_state();
        let state = area_state.borrow();
        let ref area = state.area;

        if *self.cache_invalid.borrow() {
            renderer.deregister_texture(&self.current_area_id.borrow());

            trace!("Register cached tiles texture for '{}'", area.id);
            renderer.register_texture(&area.id, self.buffer.borrow().clone()
                                      , TextureMinFilter::Nearest, TextureMagFilter::Nearest);
            *self.cache_invalid.borrow_mut() = false;
            *self.current_area_id.borrow_mut() = area.id.to_string();
        }

        // let max_x = cmp::min(inner_width, area.width - widget.state.scroll_pos.x);
        // let max_y = cmp::min(inner_height, area.height - widget.state.scroll_pos.y);
        //
        // for y in 0..max_y {
        //     for x in 0..max_x {
        //         let area_x = x + widget.state.scroll_pos.x;
        //         let area_y = y + widget.state.scroll_pos.y;
        //
        //         let tile = match area.terrain.tile_at(area_x, area_y) {
        //             &None => continue,
        //             &Some(ref tile) => tile,
        //         };
        //
        //         draw_list.append(&mut DrawList::from_sprite(&tile.image_display,
        //                                                     p.x + x, p.y + y,
        //                                                     tile.width, tile.height));
        //     }
        // }

        let tex_coords: [f32; 8] = [ 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0, 0.0 ];

        let mut draw_list = DrawList::from_texture_id(&area.id, &tex_coords,
                                                      (p.x - widget.state.scroll_pos.x) as f32,
                                                      (p.y - widget.state.scroll_pos.y) as f32,
                                                      TILE_CACHE_TEXTURE_SIZE as f32 / 16.0,
                                                      TILE_CACHE_TEXTURE_SIZE as f32 / 16.0);
        draw_list.set_scale(scale_x, scale_y);
        renderer.draw(draw_list);

        let mut draw_list = DrawList::empty_sprite();
        draw_list.set_scale(scale_x, scale_y);
        draw_list.texture_mag_filter = TextureMagFilter::Nearest;

        for transition in area.transitions.iter() {
            draw_list.append(&mut DrawList::from_sprite(
                    &transition.image_display,
                    transition.from.x + p.x - widget.state.scroll_pos.x,
                    transition.from.y + p.y - widget.state.scroll_pos.y,
                    transition.size.width, transition.size.height));
        }

        for entity in state.entity_iter() {
            let entity = entity.borrow();
            draw_list.append(&mut DrawList::from_sprite(
                    &entity.actor.actor.image_display,
                    entity.location.x + p.x - widget.state.scroll_pos.x,
                    entity.location.y + p.y - widget.state.scroll_pos.y,
                    entity.size(), entity.size()));
        }

        renderer.draw(draw_list);

        if let Some(ref cursor) = *self.cursors.borrow() {
            let mut draw_list = cursor.clone();
            draw_list.set_scale(scale_x, scale_y);
            renderer.draw(draw_list);
        }

    }

    fn on_key_press(&self, widget: &Rc<RefCell<Widget>>, key: InputAction) -> bool {
        use grt::io::InputAction::*;
        match key {
           ScrollUp => widget.borrow_mut().state.scroll(0, -1),
           ScrollDown => widget.borrow_mut().state.scroll(0, 1),
           ScrollLeft => widget.borrow_mut().state.scroll(-1, 0),
           ScrollRight => widget.borrow_mut().state.scroll(1, 0),
           _ => return false,
        };
        true
    }


    fn on_mouse_release(&self, widget: &Rc<RefCell<Widget>>, kind: ClickKind) -> bool {
        self.super_on_mouse_release(widget, kind);
        let (x, y) = self.get_cursor_pos(widget);
        if x < 0 || y < 0 { return true; }

        let action_menu = ActionMenu::new(GameState::area_state(), x, y);
        if kind == ClickKind::Left {
            action_menu.fire_default_callback();
        } else if kind == ClickKind::Right {
            Widget::add_child_to(widget, Widget::with_defaults(action_menu));
        }

        true
    }

    fn on_mouse_move(&self, widget: &Rc<RefCell<Widget>>) -> bool {
        let (area_x, area_y) = self.get_cursor_pos(widget);
        let area_state = GameState::area_state();

        {
            let ref mut state = self.mouse_over.borrow_mut().state;
            state.clear_text_params();
            state.add_text_param(&format!("{}", area_x));
            state.add_text_param(&format!("{}", area_y));
        }
        self.mouse_over.borrow_mut().invalidate_layout();

        let mut cursor_draw_list: Option<DrawList> = None;
        if let Some(entity) = area_state.borrow().get_entity_at(area_x, area_y) {
            let index = entity.borrow().index;
            let pc = GameState::pc();
            if index != pc.borrow().index {
                Widget::set_mouse_over(widget, Label::new(&entity.borrow().actor.actor.id));
                let sprite = &entity.borrow().size.cursor_sprite;
                let x = entity.borrow().location.x;
                let y = entity.borrow().location.y;
                let size = entity.borrow().size();
                cursor_draw_list = Some(DrawList::from_sprite(sprite, x, y, size, size));
            }
        }

        self.clear_cursors();
        if let Some(mut cursor_draw_list) = cursor_draw_list {
            cursor_draw_list.set_color(1.0, 0.0, 0.0, 1.0);
            self.add_cursor(cursor_draw_list);
        } else {
            let pc = GameState::pc();
            let size = pc.borrow().size();

            let (c_x, c_y) = self.get_cursor_pos_no_scroll(widget);
            let mut draw_list = DrawList::from_sprite(&pc.borrow().size.cursor_sprite,
                c_x - size / 2, c_y - size / 2, size, size);

            let action_menu = ActionMenu::new(Rc::clone(&area_state), area_x, area_y);
            if !action_menu.is_default_callback_valid() {
                draw_list.set_color(1.0, 0.0, 0.0, 1.0);
            }

            self.add_cursor(draw_list);
        }
        true
    }

    fn on_mouse_exit(&self, widget: &Rc<RefCell<Widget>>) -> bool {
        self.super_on_mouse_exit(widget);
        self.mouse_over.borrow_mut().state.clear_text_params();
        self.clear_cursors();
        true
    }
}
