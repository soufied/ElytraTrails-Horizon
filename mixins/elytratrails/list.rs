fn official_elytratrails_listed(mods: &mut Vec<crate::game::native_hooks::NativeMod>) {
    use horizon_client_render::gfx::native_render::elytratrails as m;
    mods.push(crate::game::native_hooks::NativeMod {
        id: m::ID.into(),
        title: format!("{} {}", m::NAME, m::VERSION),
        description: m::DESCRIPTION.into(),
        keys: Vec::new(),
    });
}
