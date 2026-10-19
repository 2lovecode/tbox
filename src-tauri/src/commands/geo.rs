/// 坐标系换算与距离。算法与原 `CoordinateTools.vue` 一致。

#[tauri::command]
pub fn geo_convert(lat: f64, lng: f64, from: String, to: String) -> Result<String, String> {
    let (new_lng, new_lat) = convert_coordinate(lng, lat, &from, &to);
    Ok(format!("{new_lat:.6}, {new_lng:.6}"))
}

#[tauri::command]
pub fn geo_distance(
    lat1: f64,
    lng1: f64,
    lat2: f64,
    lng2: f64,
    system: String,
    method: String,
) -> Result<serde_json::Value, String> {
    for (name, v, lo, hi) in [
        ("纬度", lat1, -90.0, 90.0),
        ("纬度", lat2, -90.0, 90.0),
        ("经度", lng1, -180.0, 180.0),
        ("经度", lng2, -180.0, 180.0),
    ] {
        if v < lo || v > hi {
            return Err(format!("{name}超出范围"));
        }
    }
    let (mut a_lng, mut a_lat, mut b_lng, mut b_lat) = (lng1, lat1, lng2, lat2);
    if system != "WGS84" {
        let a = convert_coordinate(lng1, lat1, &system, "WGS84");
        let b = convert_coordinate(lng2, lat2, &system, "WGS84");
        a_lng = a.0;
        a_lat = a.1;
        b_lng = b.0;
        b_lat = b.1;
    }
    let meters = match method.as_str() {
        "cosine" => cosine_distance(a_lat, a_lng, b_lat, b_lng) * 1000.0,
        "vincenty" => vincenty_distance(a_lat, a_lng, b_lat, b_lng),
        _ => haversine_distance(a_lat, a_lng, b_lat, b_lng) * 1000.0,
    };
    Ok(serde_json::json!({
        "km": format!("{:.3}", meters / 1000.0),
        "m": format!("{}", meters.round() as i64),
    }))
}

fn convert_coordinate(lng: f64, lat: f64, from: &str, to: &str) -> (f64, f64) {
    if from == to {
        return (lng, lat);
    }
    let wgs = match from {
        "GCJ02" => gcj02_to_wgs84(lng, lat),
        "BD09" => {
            let (g_lng, g_lat) = bd09_to_gcj02(lng, lat);
            gcj02_to_wgs84(g_lng, g_lat)
        }
        _ => (lng, lat),
    };
    match to {
        "GCJ02" => wgs84_to_gcj02(wgs.0, wgs.1),
        "BD09" => {
            let (g_lng, g_lat) = wgs84_to_gcj02(wgs.0, wgs.1);
            gcj02_to_bd09(g_lng, g_lat)
        }
        _ => wgs,
    }
}

fn wgs84_to_gcj02(lng: f64, lat: f64) -> (f64, f64) {
    const PI: f64 = std::f64::consts::PI;
    const A: f64 = 6378245.0;
    const EE: f64 = 0.00669342162296594323;
    let mut d_lat = transform_lat(lng - 105.0, lat - 35.0);
    let mut d_lng = transform_lng(lng - 105.0, lat - 35.0);
    let rad_lat = lat / 180.0 * PI;
    let mut magic = rad_lat.sin();
    magic = 1.0 - EE * magic * magic;
    let sqrt_magic = magic.sqrt();
    d_lat = (d_lat * 180.0) / ((A * (1.0 - EE)) / (magic * sqrt_magic) * PI);
    d_lng = (d_lng * 180.0) / (A / sqrt_magic * rad_lat.cos() * PI);
    (lng + d_lng, lat + d_lat)
}

fn gcj02_to_wgs84(lng: f64, lat: f64) -> (f64, f64) {
    let (g_lng, g_lat) = wgs84_to_gcj02(lng, lat);
    (lng * 2.0 - g_lng, lat * 2.0 - g_lat)
}

fn gcj02_to_bd09(lng: f64, lat: f64) -> (f64, f64) {
    let z = (lng * lng + lat * lat).sqrt() + 0.00002 * (lat * std::f64::consts::PI).sin();
    let theta = lat.atan2(lng) + 0.000003 * (lng * std::f64::consts::PI).cos();
    (z * theta.cos() + 0.0065, z * theta.sin() + 0.006)
}

fn bd09_to_gcj02(lng: f64, lat: f64) -> (f64, f64) {
    let x = lng - 0.0065;
    let y = lat - 0.006;
    let z = (x * x + y * y).sqrt() + 0.00002 * (y * std::f64::consts::PI).sin();
    let theta = y.atan2(x) + 0.000003 * (x * std::f64::consts::PI).cos();
    (z * theta.cos(), z * theta.sin())
}

fn transform_lat(x: f64, y: f64) -> f64 {
    const PI: f64 = std::f64::consts::PI;
    let mut ret = -100.0 + 2.0 * x + 3.0 * y + 0.2 * y * y + 0.1 * x * y + 0.2 * x.abs().sqrt();
    ret += (20.0 * (6.0 * x * PI).sin() + 20.0 * (2.0 * x * PI).sin()) * 2.0 / 3.0;
    ret += (20.0 * (y * PI).sin() + 40.0 * (y / 3.0 * PI).sin()) * 2.0 / 3.0;
    ret += (160.0 * (y / 12.0 * PI).sin() + 320.0 * (y * PI / 30.0).sin()) * 2.0 / 3.0;
    ret
}

fn transform_lng(x: f64, y: f64) -> f64 {
    const PI: f64 = std::f64::consts::PI;
    let mut ret = 300.0 + x + 2.0 * y + 0.1 * x * x + 0.1 * x * y + 0.1 * x.abs().sqrt();
    ret += (20.0 * (6.0 * x * PI).sin() + 20.0 * (2.0 * x * PI).sin()) * 2.0 / 3.0;
    ret += (20.0 * (x * PI).sin() + 40.0 * (x / 3.0 * PI).sin()) * 2.0 / 3.0;
    ret += (150.0 * (x / 12.0 * PI).sin() + 300.0 * (x / 30.0 * PI).sin()) * 2.0 / 3.0;
    ret
}

fn to_radians(degrees: f64) -> f64 {
    degrees * std::f64::consts::PI / 180.0
}

fn haversine_distance(lat1: f64, lng1: f64, lat2: f64, lng2: f64) -> f64 {
    let r = 6371.0;
    let d_lat = to_radians(lat2 - lat1);
    let d_lng = to_radians(lng2 - lng1);
    let a = (d_lat / 2.0).sin().powi(2)
        + to_radians(lat1).cos() * to_radians(lat2).cos() * (d_lng / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
    r * c
}

fn cosine_distance(lat1: f64, lng1: f64, lat2: f64, lng2: f64) -> f64 {
    let r = 6371.0;
    let d_lng = to_radians(lng2 - lng1);
    let a = to_radians(lat1).sin() * to_radians(lat2).sin()
        + to_radians(lat1).cos() * to_radians(lat2).cos() * d_lng.cos();
    r * a.clamp(-1.0, 1.0).acos()
}

fn vincenty_distance(lat1: f64, lng1: f64, lat2: f64, lng2: f64) -> f64 {
    let a = 6378137.0;
    let f = 1.0 / 298.257223563;
    let b = a * (1.0 - f);
    let l = to_radians(lng2 - lng1);
    let u1 = ((1.0 - f) * to_radians(lat1).tan()).atan();
    let u2 = ((1.0 - f) * to_radians(lat2).tan()).atan();
    let sin_u1 = u1.sin();
    let cos_u1 = u1.cos();
    let sin_u2 = u2.sin();
    let cos_u2 = u2.cos();
    let mut lambda = l;
    let mut lambda_p;
    let mut sin_sigma;
    let mut cos_sigma;
    let mut sigma;
    let mut cos_sq_alpha;
    let mut cos2_sigma_m;
    let mut iter = 0;
    loop {
        lambda_p = lambda;
        let sin_lambda = lambda.sin();
        let cos_lambda = lambda.cos();
        sin_sigma = ((cos_u2 * sin_lambda).powi(2)
            + (cos_u1 * sin_u2 - sin_u1 * cos_u2 * cos_lambda).powi(2))
        .sqrt();
        if sin_sigma == 0.0 {
            return 0.0;
        }
        cos_sigma = sin_u1 * sin_u2 + cos_u1 * cos_u2 * cos_lambda;
        sigma = sin_sigma.atan2(cos_sigma);
        let sin_alpha = cos_u1 * cos_u2 * sin_lambda / sin_sigma;
        cos_sq_alpha = 1.0 - sin_alpha * sin_alpha;
        cos2_sigma_m = if cos_sq_alpha != 0.0 {
            cos_sigma - 2.0 * sin_u1 * sin_u2 / cos_sq_alpha
        } else {
            0.0
        };
        let c = f / 16.0 * cos_sq_alpha * (4.0 + f * (4.0 - 3.0 * cos_sq_alpha));
        lambda = l
            + (1.0 - c)
                * f
                * sin_alpha
                * (sigma
                    + c * sin_sigma
                        * (cos2_sigma_m + c * cos_sigma * (-1.0 + 2.0 * cos2_sigma_m * cos2_sigma_m)));
        iter += 1;
        if (lambda - lambda_p).abs() <= 1e-12 || iter > 100 {
            break;
        }
    }
    let u_sq = cos_sq_alpha * (a * a - b * b) / (b * b);
    let aa = 1.0 + u_sq / 16384.0 * (4096.0 + u_sq * (-768.0 + u_sq * (320.0 - 175.0 * u_sq)));
    let bb = u_sq / 1024.0 * (256.0 + u_sq * (-128.0 + u_sq * (74.0 - 47.0 * u_sq)));
    let delta_sigma = bb
        * sin_sigma
        * (cos2_sigma_m
            + bb / 4.0
                * (cos_sigma * (-1.0 + 2.0 * cos2_sigma_m * cos2_sigma_m)
                    - bb / 6.0
                        * cos2_sigma_m
                        * (-3.0 + 4.0 * sin_sigma * sin_sigma)
                        * (-3.0 + 4.0 * cos2_sigma_m * cos2_sigma_m)));
    b * aa * (sigma - delta_sigma)
}
