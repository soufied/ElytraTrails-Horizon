fn elytratrails_net_from_server(_game: &mut crate::game::Game<'_>, channel: &str, data: &[u8]) {
    use horizon_client_render::gfx::native_render::elytratrails as m;
    if channel != m::SYNC_CHANNEL {
        return;
    }
    if let Some((name, color)) = m::decode_remote(data) {
        m::set_remote_color(&name, color);
    }
}
