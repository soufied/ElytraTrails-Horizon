fn elytratrails_draw(
    game: &crate::game::Game<'_>,
    mob: &crate::game::native_mob_draw::NativeMobRender<'_>,
    who: Option<&crate::game::native_mob_draw::NativePlayerRender<'_>>,
    draw: &mut crate::game::native_block_entity_draw::BlockEntityDraw,
) {
    use crate::game::official_elytratrails as m;
    let Some(who) = who else {
        return;
    };
    let options = game.elytratrails_options();
    if !options.shows(who.local) {
        return;
    }
    let segments = m::segments(mob.id, std::time::Instant::now(), options.length.duration());
    if segments.is_empty() {
        return;
    }
    let color = if who.local {
        options.color
    } else {
        m::remote_color(who.name).unwrap_or(m::Color::Ice)
    };
    let tint = color.tint();
    let width = options.width.factor();
    for (a, b, scale) in segments {
        for corners in m::sheets(a, b, scale * width) {
            game.native_draw_quad(draw, m::SPRITE, corners, [0.0, 0.0, 16.0, 16.0], m::fade(tint, scale));
        }
    }
}
