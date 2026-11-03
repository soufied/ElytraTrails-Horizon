fn elytratrails_left(player: u128, _name: &str) {
    use horizon_client_render::gfx::native_render::elytratrails as m;
    m::server_left(player);
}
