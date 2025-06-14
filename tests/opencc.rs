use std::path::PathBuf;

use opencc_rust_windows::{DefaultConfig, OpenCC};

fn get_config_path(config: DefaultConfig) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("opencc");
    path.push(config.get_file_name());
    path
}

#[test]
fn tw2sp() {
    let config_path = get_config_path(DefaultConfig::TW2SP);
    let opencc = OpenCC::new(config_path).unwrap(); // 现在 new() 接收的是一个有效的路径
    assert_eq!(
        "凉风有讯，秋月无边，亏我思娇的情绪好比度日如年。虽然我不是玉树临风，潇洒倜傥，\
         但我有广阔的胸襟，加强劲的臂弯。",
        &opencc
            .convert(
                "涼風有訊，秋月無邊，虧我思嬌的情緒好比度日如年。雖然我不是玉樹臨風，瀟灑倜儻，\
             但我有廣闊的胸襟，加強勁的臂彎。"
            )
            .unwrap()
    );
}

#[test]
fn tw2sp_to_buffer() {
    let mut s = String::from("涼風有訊，秋月無邊，虧我思嬌的情緒好比度日如年。");
    let config_path = get_config_path(DefaultConfig::TW2SP);

    let opencc = OpenCC::new(config_path).unwrap(); // 现在 new() 接收的是一个有效的路径

    let result = opencc.convert_append(
        "雖然我不是玉樹臨風，瀟灑倜儻，但我有廣闊的胸襟，加強勁的臂彎。",
        &mut s,
    );

    assert!(result.is_ok(), "convert_append should not fail");

    assert_eq!(
        "涼風有訊，秋月無邊，虧我思嬌的情緒好比度日如年。虽然我不是玉树临风，潇洒倜傥，\
         但我有广阔的胸襟，加强劲的臂弯。",
        &s
    );
}

#[test]
fn s2twp() {
    let config_path = get_config_path(DefaultConfig::S2TWP);
    let opencc = OpenCC::new(config_path).unwrap();
    assert_eq!(
        "涼風有訊，秋月無邊，虧我思嬌的情緒好比度日如年。雖然我不是玉樹臨風，瀟灑倜儻，\
         但我有廣闊的胸襟，加強勁的臂彎。",
        &opencc
            .convert(
                "凉风有讯，秋月无边，亏我思娇的情绪好比度日如年。虽然我不是玉树临风，潇洒倜傥，\
             但我有广阔的胸襟，加强劲的臂弯。"
            )
            .unwrap()
    );
}

#[test]
fn s2twp_to_buffer() {
    let mut s = String::from("凉风有讯，秋月无边，亏我思娇的情绪好比度日如年。");

    let config_path = get_config_path(DefaultConfig::S2TWP);
    let opencc = OpenCC::new(config_path).unwrap();

    let result = opencc.convert_append(
        "虽然我不是玉树临风，潇洒倜傥，但我有广阔的胸襟，加强劲的臂弯。",
        &mut s,
    );

    assert!(result.is_ok(), "convert_append should not fail");

    assert_eq!(
        "凉风有讯，秋月无边，亏我思娇的情绪好比度日如年。雖然我不是玉樹臨風，瀟灑倜儻，\
         但我有廣闊的胸襟，加強勁的臂彎。",
        &s
    );
}
