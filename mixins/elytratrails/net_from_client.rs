fn elytratrails_net_from_client(
    game: &mut crate::game::Game<'_>,
    player: u128,
    channel: &str,
    data: &[u8],
) {
    use horizon_client_render::gfx::native_render::elytratrails as m;
    if channel != m::SYNC_CHANNEL {
        return;
    }
    if let Some((to, message)) = m::server_colour(player, data) {
        for other in to {
            let _ = game.native_net_send_to_player(other, m::SYNC_CHANNEL, &message);
        }
    }
}
