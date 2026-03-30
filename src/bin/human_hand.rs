//! Stylized anatomical hand: palm, wrist, four fingers + thumb as primitives + groups.
//! Run: `cargo run --bin human_hand` → `media/images_ppm/human_hand.ppm`

use ray_tracing_challenge_rs::camera::Camera;
use ray_tracing_challenge_rs::canvas::Color;
use ray_tracing_challenge_rs::cylinder::Cylinder;
use ray_tracing_challenge_rs::group::Group;
use ray_tracing_challenge_rs::light::PointLight;
use ray_tracing_challenge_rs::matrix::Matrix;
use ray_tracing_challenge_rs::plane::Plane;
use ray_tracing_challenge_rs::shape::Shape;
use ray_tracing_challenge_rs::sphere::Sphere;
use ray_tracing_challenge_rs::transformation::{
    rotation_x, rotation_y, rotation_z, scaling, translation, view_transform,
};
use ray_tracing_challenge_rs::tuple::Tuple;
use ray_tracing_challenge_rs::world::World;
use std::f64::consts::{FRAC_PI_2, FRAC_PI_3, FRAC_PI_4, FRAC_PI_6};

// Warm studio skin tones (not photoreal — painterly).
const SKIN_BASE: Color = Color {
    red: 0.94,
    green: 0.78,
    blue: 0.68,
};
const SKIN_SHADOW: Color = Color {
    red: 0.82,
    green: 0.62,
    blue: 0.52,
};
const NAIL: Color = Color {
    red: 0.98,
    green: 0.90,
    blue: 0.85,
};
const RING_METAL: Color = Color {
    red: 0.85,
    green: 0.72,
    blue: 0.35,
};

fn tint_skin(base: &Color, dr: f64, dg: f64, db: f64) -> Color {
    Color::new(
        (base.red + dr).clamp(0.0, 1.0),
        (base.green + dg).clamp(0.0, 1.0),
        (base.blue + db).clamp(0.0, 1.0),
    )
}

/// Softer than “plastic toy” — subsurface-ish bounce via slightly higher ambient + lower spec.
fn apply_flesh_mat(m: &mut ray_tracing_challenge_rs::material::Material, color: Color, spec: f64) {
    m.color = color;
    m.ambient = 0.085;
    m.diffuse = 0.78;
    m.specular = spec;
    m.shininess = 32.0;
}

/// Bone in finger/thumb local space: +Z along phalanx after `base`.
fn bone_cylinder_with_base(
    base: Matrix,
    radius: f64,
    length: f64,
    color: Color,
) -> Cylinder {
    let mut c = Cylinder::new();
    c.minimum = 0.0;
    c.maximum = 1.0;
    c.closed = true;
    let oriented = &rotation_x(FRAC_PI_2) * &scaling(radius, length, radius);
    c.set_transform(&base * &oriented);
    apply_flesh_mat(c.material_mut(), color, 0.12);
    c
}

/// Fingertip pad + nail plane; `tip_anchor` places the distal DIP joint (end of distal phalanx base).
fn add_fingertip_nail(
    finger_root: &mut Group,
    tip_anchor: Matrix,
    r: f64,
    skin: Color,
    with_nail: bool,
) {
    let mut tip = Sphere::new();
    tip.set_transform(
        &( &tip_anchor * &translation(0.0, 0.0, r * 0.14) ) * &scaling(r * 1.04, r * 1.04, r * 1.04),
    );
    apply_flesh_mat(tip.material_mut(), skin, 0.14);
    finger_root.add_child(Box::new(tip));

    if with_nail {
        let mut nail = Sphere::new();
        nail.set_transform(
            &( &tip_anchor * &translation(0.0, 0.016, r * 0.62 + r * 0.15) )
                * &scaling(r * 0.88, r * 0.20, r * 0.72),
        );
        nail.material_mut().color = NAIL;
        nail.material_mut().ambient = 0.06;
        nail.material_mut().diffuse = 0.75;
        nail.material_mut().specular = 0.42;
        nail.material_mut().shininess = 85.0;
        finger_root.add_child(Box::new(nail));
    }
}

fn add_joint_pad(
    finger_root: &mut Group,
    base: &Matrix,
    y_offset: f64,
    radius: f64,
    skin: Color,
) {
    let mut pad = Sphere::new();
    let t = &(base * &translation(0.0, y_offset, 0.0)) * &scaling(radius, radius * 0.65, radius);
    pad.set_transform(t);
    apply_flesh_mat(
        pad.material_mut(),
        tint_skin(&skin, -0.04, -0.03, -0.02),
        0.08,
    );
    finger_root.add_child(Box::new(pad));
}

/// Three phalanges with PIP / DIP flexion, knuckle pad, anatomical length ratios (prox > mid > dist).
fn add_finger(
    finger_root: &mut Group,
    length: f64,
    thick: f64,
    skin: Color,
    with_nail: bool,
    curl_pip: f64,
    curl_dip: f64,
) {
    let r = thick * 0.5;
    let p_len = length * 0.43;
    let m_len = length * 0.32;
    let d_len = length * 0.22;

    // MCP bulge — nudged into palm so the digit reads “rooted” not floating
    let mut kn = Sphere::new();
    kn.set_transform(
        &translation(0.0, -0.028 * (thick / 0.1).clamp(0.85, 1.15), -0.048)
            * &scaling(r * 0.58, r * 0.42, r * 0.58),
    );
    apply_flesh_mat(kn.material_mut(), tint_skin(&skin, -0.06, -0.05, -0.04), 0.08);
    finger_root.add_child(Box::new(kn));

    let prox = bone_cylinder_with_base(
        translation(0.0, 0.0, 0.0),
        r * 0.97,
        p_len,
        skin.clone(),
    );
    finger_root.add_child(Box::new(prox));

    let b_mid = &translation(0.0, 0.0, p_len) * &rotation_x(-curl_pip);
    let mid = bone_cylinder_with_base(b_mid.clone(), r * 0.90, m_len, skin.clone());
    finger_root.add_child(Box::new(mid));
    let b_pip = &translation(0.0, 0.0, p_len);
    add_joint_pad(finger_root, b_pip, -0.006, r * 0.42, skin.clone());

    let b_mid_end = &b_mid * &translation(0.0, 0.0, m_len);
    let b_dist = &b_mid_end * &rotation_x(-curl_dip);
    let dist = bone_cylinder_with_base(
        b_dist.clone(),
        r * 0.82,
        d_len,
        tint_skin(&skin, -0.02, -0.02, -0.015),
    );
    finger_root.add_child(Box::new(dist));
    add_joint_pad(finger_root, &b_mid_end, -0.004, r * 0.34, skin.clone());

    let tip_anchor = &b_dist * &translation(0.0, 0.0, d_len);
    add_fingertip_nail(finger_root, tip_anchor, r, skin, with_nail);
}

fn build_hand() -> Group {
    let mut hand = Group::new();
    // Palm-up presentation: slight world tilt
    hand.set_transform(
        &( &translation(0.0, 0.0, 0.0) * &rotation_y(FRAC_PI_6) ) * &rotation_x(-FRAC_PI_6),
    );

    // Palm: wider + flatter “metacarpal deck” (less ball, more paddle / trapezoid read)
    let mut palm = Sphere::new();
    palm.set_transform(&translation(0.0, 0.52, -0.02) * &scaling(0.46, 0.086, 0.36));
    apply_flesh_mat(palm.material_mut(), SKIN_BASE.clone(), 0.10);
    hand.add_child(Box::new(palm));

    // Thenar eminence — large mass from wrist to thumb CMC (fills radial gutter)
    let mut thenar = Sphere::new();
    thenar.set_transform(&translation(-0.22, 0.478, -0.11) * &scaling(0.19, 0.10, 0.20));
    apply_flesh_mat(thenar.material_mut(), tint_skin(&SKIN_BASE, -0.05, -0.04, -0.03), 0.09);
    hand.add_child(Box::new(thenar));

    let mut hypothenar = Sphere::new();
    hypothenar.set_transform(&translation(0.21, 0.498, -0.02) * &scaling(0.11, 0.055, 0.13));
    apply_flesh_mat(hypothenar.material_mut(), tint_skin(&SKIN_BASE, -0.05, -0.04, -0.03), 0.09);
    hand.add_child(Box::new(hypothenar));

    // Heel + carpal “bridge” softens palm–wrist junction (hides sharp cylinder cut)
    let mut heel = Sphere::new();
    heel.set_transform(&translation(0.0, 0.465, -0.20) * &scaling(0.20, 0.075, 0.18));
    apply_flesh_mat(heel.material_mut(), SKIN_SHADOW.clone(), 0.08);
    hand.add_child(Box::new(heel));

    let mut carpal_blend = Sphere::new();
    carpal_blend.set_transform(&translation(-0.06, 0.42, -0.33) * &scaling(0.24, 0.065, 0.16));
    apply_flesh_mat(
        carpal_blend.material_mut(),
        tint_skin(&SKIN_SHADOW, 0.04, 0.03, 0.03),
        0.07,
    );
    hand.add_child(Box::new(carpal_blend));

    // --- Wrist (tilted slightly to tuck under palm ellipsoid) ---
    let mut wrist = Cylinder::new();
    wrist.minimum = 0.0;
    wrist.maximum = 1.0;
    wrist.closed = true;
    wrist.set_transform(
        &( &translation(0.0, 0.375, -0.445) * &rotation_x(FRAC_PI_4 * 0.88) )
            * &scaling(0.115, 0.30, 0.115),
    );
    apply_flesh_mat(wrist.material_mut(), SKIN_SHADOW.clone(), 0.09);
    hand.add_child(Box::new(wrist));

    let mut forearm = Cylinder::new();
    forearm.minimum = 0.0;
    forearm.maximum = 1.0;
    forearm.closed = true;
    forearm.set_transform(
        &( &translation(0.0, 0.31, -0.72) * &rotation_x(FRAC_PI_4 * 0.86) )
            * &scaling(0.125, 0.42, 0.125),
    );
    apply_flesh_mat(
        forearm.material_mut(),
        tint_skin(&SKIN_SHADOW, -0.01, -0.01, -0.01),
        0.08,
    );
    hand.add_child(Box::new(forearm));

    // Knuckle line: (x, z, len, thick, splay_z, converge_y, nail, PIP, DIP).
    // Fingers slightly converge toward middle finger and have a mild transverse arch in z.
    let finger_specs = [
        (-0.26, 0.205, 0.54, 0.102, 0.15, 0.08, true, 0.24, 0.36),  // index
        (-0.09, 0.228, 0.60, 0.105, 0.08, 0.02, true, 0.22, 0.34),  // middle
        (0.09, 0.222, 0.56, 0.100, -0.07, -0.02, true, 0.23, 0.35), // ring
        (0.25, 0.194, 0.45, 0.090, -0.17, -0.08, true, 0.28, 0.40), // pinky
    ];

    for (fx, fz, flen, fthick, z_splay, y_conv, nail, curl_p, curl_d) in finger_specs {
        let mut fg = Group::new();
        // MCPs slightly embedded in palm deck (lower Y, −Z) so cylinders aren’t “hovering”
        fg.set_transform(
            &( &( &translation(fx, 0.552, fz) * &rotation_y(y_conv) ) * &rotation_x(-0.14 - flen * 0.062) )
                * &rotation_z(z_splay),
        );
        let skin = tint_skin(
            &SKIN_BASE,
            fx * 0.035,
            -(fx.abs() * 0.025),
            -fx.abs() * 0.012,
        );
        add_finger(&mut fg, flen, fthick, skin, nail, curl_p, curl_d);
        hand.add_child(Box::new(fg));
    }

    // Webbing between fingers to soften hard cylinder starts.
    let webs = [
        (-0.175, 0.546, 0.185, 0.070, 0.020, 0.060),
        (0.000, 0.553, 0.202, 0.070, 0.020, 0.060),
        (0.170, 0.548, 0.190, 0.065, 0.020, 0.055),
    ];
    for (wx, wy, wz, sx, sy, sz) in webs {
        let mut w = Sphere::new();
        w.set_transform(&translation(wx, wy, wz) * &scaling(sx, sy, sz));
        apply_flesh_mat(
            w.material_mut(),
            tint_skin(&SKIN_BASE, -0.03, -0.02, -0.015),
            0.08,
        );
        hand.add_child(Box::new(w));
    }

    // Subtle dorsal tendons.
    let tendon_specs = [(-0.10, 0.525, -0.005), (0.08, 0.528, -0.010)];
    for (tx, ty, rz) in tendon_specs {
        let mut t = Cylinder::new();
        t.minimum = 0.0;
        t.maximum = 1.0;
        t.closed = true;
        t.set_transform(
            &( &( &translation(tx, ty, -0.005) * &rotation_x(FRAC_PI_2) ) * &rotation_z(rz) )
                * &scaling(0.011, 0.30, 0.011),
        );
        apply_flesh_mat(
            t.material_mut(),
            tint_skin(&SKIN_BASE, -0.08, -0.07, -0.06),
            0.06,
        );
        hand.add_child(Box::new(t));
    }

    // Gold band on ring finger (follows MCP frame + proximal shaft)
    let ring_base = &( &( &translation(0.09, 0.552, 0.222) * &rotation_y(-0.02) ) * &rotation_x(-0.14 - 0.56 * 0.062) )
        * &rotation_z(-0.07);
    let mut ring = Cylinder::new();
    ring.minimum = 0.0;
    ring.maximum = 1.0;
    ring.closed = true;
    ring.set_transform(
        &( &( &ring_base * &translation(0.0, 0.0, 0.17) ) * &rotation_x(FRAC_PI_2) )
            * &scaling(0.046, 0.026, 0.046),
    );
    ring.material_mut().color = RING_METAL;
    ring.material_mut().specular = 0.9;
    ring.material_mut().shininess = 220.0;
    hand.add_child(Box::new(ring));

    // --- Thumb: two-part hierarchy fixes orientation ---
    // `thumb_shell` = CMC on radial rim only (translation). `thumb_orient` = rotation only so local +Z
    // runs from radial border toward the index/MCP line (opposition), same convention as fingers.
    // Bones stay in `thumb_orient` space — no mixed Ry·Rz·Rx on one matrix splintering segments.
    let mut thumb_shell = Group::new();
    // Radial edge (-X), on palm deck, slightly toward fingertips (+Z) — not on palm “screen” plane
    thumb_shell.set_transform(translation(-0.305, 0.515, 0.022));

    let mut thumb_orient = Group::new();
    // R_x * R_y: first R_y swings default +Z toward +X (into palm); R_x lifts toward +Y (out of palm).
    // Tuned so one continuous chain reads as one thumb, not scattered cylinders.
    thumb_orient.set_transform(
        &( &rotation_x(-0.18) * &rotation_y(0.62) ) * &rotation_z(0.10),
    );

    // CMC pad — sits at origin of oriented chain (thenar side)
    let mut t_kn = Sphere::new();
    t_kn.set_transform(
        &translation(0.0, -0.022, -0.028) * &scaling(0.095, 0.062, 0.095),
    );
    apply_flesh_mat(
        t_kn.material_mut(),
        tint_skin(&SKIN_BASE, -0.05, -0.04, -0.035),
        0.08,
    );
    thumb_orient.add_child(Box::new(t_kn));

    let tm1 = 0.116;
    let tm2 = 0.122;
    let tm3 = 0.086;
    let tc1 = 0.22;
    let tc2 = 0.30;
    let t_r = 0.082;

    let t_meta = bone_cylinder_with_base(
        translation(0.0, 0.0, 0.0),
        t_r,
        tm1,
        tint_skin(&SKIN_BASE, -0.03, -0.02, -0.02),
    );
    thumb_orient.add_child(Box::new(t_meta));

    let t_b2 = &translation(0.0, 0.0, tm1) * &rotation_x(-tc1);
    let t_prox = bone_cylinder_with_base(
        t_b2.clone(),
        t_r * 0.93,
        tm2,
        SKIN_BASE.clone(),
    );
    thumb_orient.add_child(Box::new(t_prox));

    let t_b2e = &t_b2 * &translation(0.0, 0.0, tm2);
    let t_b3 = &t_b2e * &rotation_x(-tc2);
    let t_dist = bone_cylinder_with_base(
        t_b3.clone(),
        t_r * 0.86,
        tm3,
        tint_skin(&SKIN_BASE, -0.02, -0.02, -0.015),
    );
    thumb_orient.add_child(Box::new(t_dist));

    let t_tip_anchor = &t_b3 * &translation(0.0, 0.0, tm3);
    add_fingertip_nail(&mut thumb_orient, t_tip_anchor, t_r, SKIN_BASE.clone(), true);

    thumb_shell.add_child(Box::new(thumb_orient));
    hand.add_child(Box::new(thumb_shell));

    hand
}

fn main() {
    let mut world = World::new();

    // Table / horizon
    let mut floor = Plane::new();
    floor.set_transform(translation(0.0, 0.0, 0.0));
    floor.material_mut().color = Color::new(0.16, 0.14, 0.20);
    floor.material_mut().specular = 0.08;
    floor.material_mut().reflective = 0.12;
    world.add_shape(floor);

    let mut backdrop = Plane::new();
    backdrop.set_transform(
        &( &translation(0.0, 0.0, 2.9) * &rotation_x(FRAC_PI_2) ) * &rotation_y(0.03),
    );
    backdrop.material_mut().color = Color::new(0.17, 0.16, 0.26);
    backdrop.material_mut().ambient = 0.20;
    backdrop.material_mut().diffuse = 0.72;
    backdrop.material_mut().specular = 0.04;
    world.add_shape(backdrop);

    world.add_shape(build_hand());

    // Lighting: strong key + softer fills to reduce harsh “digital” contrast / banding in shadow
    world.lights = vec![
        PointLight::new(
            Tuple::point(-6.0, 8.0, -4.0),
            Color::new(0.88, 0.82, 0.74),
        ),
        PointLight::new(
            Tuple::point(8.0, 5.0, 2.0),
            Color::new(0.42, 0.48, 0.58),
        ),
        PointLight::new(
            Tuple::point(0.0, 4.5, 5.5),
            Color::new(0.38, 0.38, 0.42),
        ),
        PointLight::new(
            Tuple::point(-3.0, 2.0, 4.0),
            Color::new(0.22, 0.22, 0.24),
        ),
    ];

    let mut camera = Camera::new(1000, 700, FRAC_PI_3);
    camera.transform = view_transform(
        &Tuple::point(-1.02, 1.10, -2.70),
        &Tuple::point(0.0, 0.52, 0.10),
        &Tuple::vector(0.0, 1.0, 0.0),
    );

    println!("Rendering stylized hand (this may take a minute)...");
    let canvas = camera.render(&world);
    let ppm = canvas.canvas_to_ppm();
    let path = "media/images_ppm/human_hand.ppm";
    std::fs::write(path, ppm).unwrap_or_else(|e| panic!("Failed to write {path}: {e}"));
    println!("Saved to {path}");
}
