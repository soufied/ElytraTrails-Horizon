fn elytratrails_settings_click(game: &mut crate::game::Game<'_>, mod_id: &str, button: usize) {
    use horizon_client_render::gfx::native_render::elytratrails as m;
    if mod_id == m::ID {
        game.elytratrails_page_click(button);
    }
}
