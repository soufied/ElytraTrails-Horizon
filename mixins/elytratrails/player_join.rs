fn elytratrails_joined(game: &mut crate::game::Game<'_>, player: u128, name: &str) {
    use horizon_client_render::gfx::native_render::elytratrails as m;
    for message in m::server_joined(player, name) {
        let _ = game.native_net_send_to_player(player, m::SYNC_CHANNEL, &message);
    }
}
