fn elytratrails_record(
    game: &crate::game::Game<'_>,
    mob: &crate::mobs::Mob,
    partial: f32,
    who: &Option<crate::game::native_mob_draw::NativePlayerRender<'_>>,
) {
    use horizon_client_render::gfx::native_render::elytratrails as m;

    fn elytratrails_wing_tips(
        game: &crate::game::Game<'_>,
        mob: &crate::mobs::Mob,
        partial: f32,
        feet: [f64; 3],
        body_yaw: f32,
    ) -> Option<[[f64; 3]; 2]> {
        use horizon_client_render::gfx::native_render::elytratrails as m;
        let model = game.armor_stand_assets.player.get("elytra")?;
        let angles = mob.extra.elytra.angles(partial);
        let crouching = mob.extra.crouching;
        let origin = glam::Mat4::from_translation(glam::Vec3::new(
            feet[0] as f32,
            feet[1] as f32,
            feet[2] as f32,
        ));
        let base = horizon_client::mobs::render::base_transform_at(mob, partial, origin)
            * glam::Mat4::from_translation(glam::Vec3::new(0.0, 0.0, 0.125));
        let pose = |part: &horizon_client::mobs::Part| {
            let mut p = part.pose;
            if part.name == "left_wing" || part.name == "right_wing" {
                let invert = if part.name == "left_wing" { 1.0 } else { -1.0 };
                p[1] = if crouching { 3.0 } else { 0.0 };
                p[3] = angles[0];
                p[4] = invert * angles[1];
                p[5] = invert * angles[2];
            }
            p
        };
        let mut verts = Vec::new();
        horizon_client::mobs::render_model(model, base, &pose, [0.0; 4], 0, 0, &mut verts);

        let root = [0.0_f32, m::ROOT_UP as f32, 0.0];
        let mut best: [Option<(f32, [f64; 3])>; 2] = [None, None];
        for v in &verts {
            let world = [f64::from(v.pos[0]), f64::from(v.pos[1]), f64::from(v.pos[2])];
            let local = m::to_local(world, feet, body_yaw);
            let side = if local[0] >= 0.0 { 0 } else { 1 };
            let d = (0..3).map(|i| (local[i] - root[i]).powi(2)).sum::<f32>();
            if best[side].is_none_or(|(bd, _)| d > bd) {
                best[side] = Some((d, world));
            }
        }
        Some([best[0]?.1, best[1]?.1])
    }

    let Some(who) = who else {
        return;
    };
    if !game.elytratrails_options().shows(who.local) {
        return;
    }
    let a = f64::from(partial);
    let feet = [
        mob.previous.x + (mob.position.x - mob.previous.x) * a,
        mob.previous.y + (mob.position.y - mob.previous.y) * a,
        mob.previous.z + (mob.position.z - mob.previous.z) * a,
    ];
    let body_yaw = mob.previous_body_yaw
        + horizon_client::mobs::living::wrap(mob.body_yaw - mob.previous_body_yaw) * partial;
    let gliding = mob.extra.fall_flying;
    let tips = if gliding {
        elytratrails_wing_tips(game, mob, partial, feet, body_yaw)
            .unwrap_or_else(|| m::wing_tips(feet, body_yaw))
    } else {
        [feet; 2]
    };
    m::record_at(
        mob.id,
        feet,
        body_yaw,
        gliding,
        tips,
        std::time::Instant::now(),
        m::Viewer {
            local: who.local,
            name: who.name,
        },
    );
}
