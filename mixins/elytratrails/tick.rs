fn elytratrails_tick(game: &mut crate::game::Game<'_>) {
    use horizon_client_render::gfx::native_render::elytratrails as m;
    let options = game.elytratrails_options();
    m::set_options(options);
    if m::sync_due(std::time::Instant::now()) {
        let _ = game.native_net_send_to_server(m::SYNC_CHANNEL, options.color.code().as_bytes());
    }
}
