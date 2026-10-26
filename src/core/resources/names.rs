// Generated once by the converter from the GameMaker project, maintained by hand since.
//! Every sprite and sound the code refers to, by the name of its file
//! (without extension) inside Textures/ or Sounds/. Constants keep the
//! original GameMaker names, so GML and Rust code can be compared side by side.

use super::{SoundId, SpriteId};

pub const SPRITE_FILES: [&str; 995] = [
    "act9",
    "aiz",
    "aiz10",
    "aiz2",
    "aiz3",
    "aiz35",
    "aiz38",
    "aiz4",
    "aiz5",
    "aiz6",
    "aiz7",
    "aiz8",
    "aiz9",
    "am",
    "am2",
    "am3",
    "darktower",
    "dotdotdot",
    "dotdotdot2",
    "dotdotdot3",
    "dt",
    "dt2",
    "dt3",
    "dt4",
    "dt5",
    "dt6",
    "greenhill",
    "greenhill2",
    "greenhill3",
    "greenhill4",
    "greenhill5",
    "greenhill6",
    "hd",
    "hd2",
    "hd3",
    "hd4",
    "heathaze_noise",
    "hn2",
    "hn22",
    "knf",
    "knf2",
    "knf3",
    "knf4",
    "knf5",
    "knf6",
    "limpcity",
    "limpcity2",
    "limpcity3",
    "majin_forest",
    "majin_forest2",
    "majin_forest3",
    "mj",
    "mj2",
    "mj3",
    "mj4",
    "mj5",
    "nap",
    "nap2",
    "nap3",
    "notperf",
    "notperf2",
    "notperf3",
    "pf",
    "rm",
    "rm2",
    "rm3",
    "test",
    "vv",
    "vv2",
    "vv3",
    "vv4",
    "vv5",
    "weed",
    "weed2",
    "weed3",
    "weed4",
    "ycr",
    "ycr2",
    "ycr3",
    "ycr4",
    "hd_colliison",
    "abyss",
    "abyss_target",
    "achivarrows",
    "achivementbox",
    "achivements",
    "achivements_new",
    "act9_dead",
    "act9_tiles_1",
    "act9_wall",
    "aiz_decor",
    "aiz_decor2",
    "aiz_fire",
    "aiz_fire2",
    "aiz_liana",
    "aiz_tiles_1",
    "aiz_tiles2_1",
    "aiz_zipline",
    "am_acid",
    "am_cloud",
    "am_collision",
    "am_face",
    "am_ooh",
    "am_tiles_1",
    "am_tiles2_1",
    "am_tiles3_1",
    "amy_attack1",
    "amy_attack2",
    "amy_balancing",
    "amy_dead",
    "amy_emotion1",
    "amy_emotion2",
    "amy_emotion3",
    "amy_fall",
    "amy_hurt",
    "amy_idle",
    "amy_jump",
    "amy_lookdown",
    "amy_lookup",
    "amy_run",
    "amy_walk",
    "amy_zipline",
    "attackgui",
    "badconnection",
    "bigring",
    "bigring_ready",
    "black",
    "blackring",
    "blackring_purple",
    "blackring_sparkle",
    "blackring_sparkle_purple",
    "block",
    "block2",
    "blood1",
    "blood2",
    "blood3",
    "bluespheres_sphere",
    "boom",
    "bspring_left",
    "bspring_right",
    "bspring_up",
    "chaos_attack1",
    "chaos_attack2",
    "chaos_balancing",
    "chaos_emotion1",
    "chaos_emotion2",
    "chaos_emotion3",
    "chaos_fall",
    "chaos_hurt",
    "chaos_idle",
    "chaos_jump",
    "chaos_liquid",
    "chaos_lookdown",
    "chaos_lookup",
    "chaos_lost",
    "chaos_lost2",
    "chaos_run",
    "chaos_sfall",
    "chaos_sidle",
    "chaos_sjump",
    "chaos_stransform",
    "chaos_stransformair",
    "chaos_stuck",
    "chaos_stuck2",
    "chaos_stun",
    "chaos_swalk",
    "chaos_walk",
    "chaos_won",
    "chaos_zipline",
    "char_bars",
    "char_info",
    "clock",
    "clockhand",
    "corpse_amy",
    "corpse_cream",
    "corpse_eggman",
    "corpse_knux",
    "corpse_tails",
    "counter",
    "counter2",
    "cream_balancing",
    "cream_dead",
    "cream_emotion1",
    "cream_emotion2",
    "cream_emotion3",
    "cream_fall",
    "cream_fly",
    "cream_hurt",
    "cream_idle",
    "cream_jump",
    "cream_lookdown",
    "cream_lookup",
    "cream_run",
    "cream_srings",
    "cream_walk",
    "cream_zipline",
    "darktower_ball",
    "darktower_fog",
    "darktower_jumpscare",
    "darktower_stalactite",
    "darktower_stalactite1",
    "darktower_stalactite2",
    "darktower_tailsol",
    "darktower_tiles_1",
    "deathtp",
    "deathtp_point",
    "deserttown_fog",
    "deserttown_hide",
    "deserttown_hide2",
    "deserttown_misc",
    "deserttown_misctiles",
    "deserttown_tiles_1",
    "deserttown_tiles2_1",
    "deserttown_tiles3_1",
    "dot_eggstatue",
    "dot_fountain",
    "dot_fountain2",
    "dot_fountain3",
    "dot_tiles_1",
    "dot_tiles1_1",
    "dust",
    "eamy_attack1",
    "eamy_attack2",
    "eamy_balancing",
    "eamy_emotion1",
    "eamy_emotion2",
    "eamy_emotion3",
    "eamy_fall",
    "eamy_hurt",
    "eamy_idle",
    "eamy_jump",
    "eamy_lookdown",
    "eamy_lookup",
    "eamy_run",
    "eamy_stun",
    "eamy_walk",
    "eamy_zipline",
    "ecream_balancing",
    "ecream_emotion1",
    "ecream_emotion2",
    "ecream_emotion3",
    "ecream_fall",
    "ecream_fly",
    "ecream_hurt",
    "ecream_idle",
    "ecream_jump",
    "ecream_lookdown",
    "ecream_lookup",
    "ecream_run",
    "ecream_srings",
    "ecream_stun",
    "ecream_walk",
    "ecream_zipline",
    "eegg_balancing",
    "eegg_djump",
    "eegg_emotion1",
    "eegg_emotion2",
    "eegg_emotion3",
    "eegg_fall",
    "eegg_hurt",
    "eegg_idle",
    "eegg_jump",
    "eegg_lookdown",
    "eegg_lookup",
    "eegg_run",
    "eegg_stun",
    "eegg_walk",
    "eegg_zipline",
    "egg_balancing",
    "egg_dead",
    "egg_djump",
    "egg_emotion1",
    "egg_emotion2",
    "egg_emotion3",
    "egg_fall",
    "egg_hurt",
    "egg_idle",
    "egg_jump",
    "egg_lookdown",
    "egg_lookup",
    "egg_run",
    "egg_walk",
    "egg_zipline",
    "eggpack",
    "eggtrack",
    "eggtrack_destroy",
    "eknux_attack1",
    "eknux_attack2",
    "eknux_balancing",
    "eknux_emotion1",
    "eknux_emotion2",
    "eknux_emotion3",
    "eknux_fall",
    "eknux_glide",
    "eknux_hurt",
    "eknux_idle",
    "eknux_jump",
    "eknux_lookdown",
    "eknux_lookup",
    "eknux_run",
    "eknux_stuck",
    "eknux_stun",
    "eknux_walk",
    "eknux_zipline",
    "electroshield",
    "eroseheart",
    "esally_attack",
    "esally_balancing",
    "esally_emotion1",
    "esally_emotion2",
    "esally_emotion3",
    "esally_fall",
    "esally_hurt",
    "esally_idle",
    "esally_jump",
    "esally_lookdown",
    "esally_lookup",
    "esally_run",
    "esally_slide",
    "esally_stun",
    "esally_walk",
    "esally_zipline",
    "etails_attack1",
    "etails_balancing",
    "etails_emotion1",
    "etails_emotion2",
    "etails_emotion3",
    "etails_fall",
    "etails_fly",
    "etails_hurt",
    "etails_idle",
    "etails_jump",
    "etails_lookdown",
    "etails_lookup",
    "etails_run",
    "etails_stun",
    "etails_tail1",
    "etails_tail2",
    "etails_walk",
    "etails_zipline",
    "exe_attack1",
    "exe_attack2",
    "exe_balancing",
    "exe_emotion",
    "exe_emotion2",
    "exe_emotion3",
    "exe_fall",
    "exe_hurt",
    "exe_idle",
    "exe_invis_balancing",
    "exe_invis_emotion",
    "exe_invis_fall",
    "exe_invis_hurt",
    "exe_invis_idle",
    "exe_invis_jump",
    "exe_invis_lookdown",
    "exe_invis_lookup",
    "exe_invis_run",
    "exe_invis_walk",
    "exe_invis_zipline",
    "exe_jump",
    "exe_lookdown",
    "exe_lookup",
    "exe_lost",
    "exe_run",
    "exe_shocked",
    "exe_walk",
    "exe_won",
    "exe_zipline",
    "exeller_attack1",
    "exeller_attack2",
    "exeller_balancing",
    "exeller_clone",
    "exeller_clone2",
    "exeller_clonearrow",
    "exeller_clonetracker",
    "exeller_clonetracker2",
    "exeller_emotion",
    "exeller_emotion2",
    "exeller_emotion3",
    "exeller_fall",
    "exeller_hurt",
    "exeller_idle",
    "exeller_jump",
    "exeller_lookdown",
    "exeller_lookup",
    "exeller_lost",
    "exeller_lost2",
    "exeller_run",
    "exeller_shocked",
    "exeller_walk",
    "exeller_won",
    "exeller_zipline",
    "exespawn",
    "exetior_attack1",
    "exetior_attack2",
    "exetior_balancing",
    "exetior_emotion",
    "exetior_emotion2",
    "exetior_emotion3",
    "exetior_fall",
    "exetior_hurt",
    "exetior_idle",
    "exetior_jump",
    "exetior_lookdown",
    "exetior_lookup",
    "exetior_lost",
    "exetior_run",
    "exetior_shocked",
    "exetior_stomp",
    "exetior_stompballs",
    "exetior_stompland",
    "exetior_walk",
    "exetior_won",
    "exetior_zipline",
    "explosion",
    "fart_dummy",
    "fartzone",
    "frozen",
    "gameover",
    "ghz_electroshock",
    "ghz_palm",
    "ghz_rain",
    "ghz_slope1",
    "ghz_slope2",
    "ghz_slope3",
    "ghz_slope4",
    "ghz_slope5",
    "ghz_tiles_1",
    "ghz_tiles2_1",
    "ghz_waterfall",
    "ghz_waterfall2",
    "ghz_wave",
    "goodperson",
    "green",
    "green293",
    "gui_amyattack",
    "gui_amyhjump",
    "gui_chaosattack",
    "gui_chaosslime",
    "gui_chaoswalldash",
    "gui_creamdash",
    "gui_creamfly",
    "gui_creamrings",
    "gui_eggdjump",
    "gui_eggshield",
    "gui_eggtrack",
    "gui_emotion_anger",
    "gui_exeattack",
    "gui_exefreejump",
    "gui_exeinvisability",
    "gui_exellerclone",
    "gui_exellerclone2",
    "gui_exetiorattack",
    "gui_exetiorring",
    "gui_knuxattack",
    "gui_knuxglide",
    "gui_sallyattack",
    "gui_sallyshield",
    "gui_tailsattack",
    "gui_tailsfly",
    "hd_crystal",
    "hd_crystalweb",
    "hd_door",
    "hd_door2",
    "hd_hide",
    "hd_spring",
    "hd_tiles_1",
    "hd_tiles2_1",
    "heal",
    "heal_part",
    "hidegui",
    "hp",
    "hp_rr",
    "hs2_box",
    "hs2_slope1",
    "hs2_slope2",
    "hs2_tiles_1",
    "indicator",
    "indicator2",
    "indicator3",
    "kindnfair_slope1",
    "kindnfair_slope2",
    "kindnfair_speedbox",
    "kindnfair_tiles_1",
    "knux_attack1",
    "knux_attack2",
    "knux_balancing",
    "knux_dead",
    "knux_emotion1",
    "knux_emotion2",
    "knux_emotion3",
    "knux_fall",
    "knux_glide",
    "knux_hurt",
    "knux_idle",
    "knux_jump",
    "knux_lookdown",
    "knux_lookup",
    "knux_marijuna",
    "knux_run",
    "knux_stuck",
    "knux_walk",
    "knux_zipline",
    "limpcity_chain1",
    "limpcity_chain2",
    "limpcity_eye",
    "limpcity_eye_chain",
    "limpcity_eye_hint",
    "limpcity_eye_recharge",
    "limpcity_eyelid",
    "limpcity_hideeye",
    "limpcity_tiles_1",
    "lobby_bars",
    "lobby_chatbox",
    "lobby_dicon",
    "lobby_exeicon",
    "lobby_exeicon2",
    "lobby_exeicon3",
    "lobby_exeicon4",
    "lobby_exeicon5",
    "lobby_icon",
    "lobby_icon_arrow",
    "lobby_icon_used",
    "logo",
    "logo_logos",
    "luigikid",
    "majin_forest_slope",
    "majin_forest_tiles_1",
    "majin_forest_wall",
    "majin_forest_waterfall",
    "mapvote",
    "marijuna_boohoo",
    "marijuna_crystal",
    "marijuna_crystal2",
    "marijuna_statue",
    "marijuna_statue2",
    "marijuna_statue3",
    "marijuna_fireball",
    "marijuna_judger",
    "marijuna_lava",
    "marijuna_lava2",
    "marijuna_lavaplatform",
    "marijuna_slopes",
    "marijuna_tiles_1",
    "marijuna_tiles2_1",
    "marijuna_torch",
    "menu_amy",
    "menu_bars",
    "menu_chaos",
    "menu_characters",
    "menu_chicons",
    "menu_cream",
    "menu_demon",
    "menu_diff",
    "menu_egg",
    "menu_escape",
    "menu_exe",
    "menu_exeller",
    "menu_exetior",
    "menu_hit",
    "menu_icon",
    "menu_knux",
    "menu_links",
    "menu_links2",
    "menu_iconframe",
    "menu_par",
    "menu_particle",
    "menu_public",
    "menu_public2",
    "menu_reb",
    "menu_redrings",
    "menu_rings",
    "menu_sally",
    "menu_skin",
    "menu_slider",
    "menu_spin",
    "menu_steal",
    "menu_sudden",
    "menu_tails",
    "menu_textbox",
    "menu_textpanel",
    "menu_waiting",
    "mercoin",
    "merfurmu",
    "merfurmu2",
    "mini_border",
    "minisnoc",
    "minisnoc_charg",
    "minisnoc_graveyard",
    "minisnoc_hit",
    "minisnoc_mcbigtasty",
    "minisnoc_spin",
    "mobile_button",
    "mobile_dpad",
    "mobile_smallbutton",
    "nap_ballmonster",
    "nap_iceblock",
    "nap_iceblock2",
    "nap_iceblock3",
    "nap_iceblock_part",
    "nap_icespike",
    "nap_snow",
    "nap_snowball",
    "nap_snowball_part",
    "nap_tails2_1",
    "nap_tiles_1",
    "nap_tiles3",
    "none",
    "notperfect_1",
    "notperfect2_1",
    "notperfect2_p2_1",
    "notperfect_p2_1",
    "menu_warning",
    "menu_warning2",
    "pet_annette",
    "pet_bals",
    "pet_catwe",
    "pet_chao",
    "pet_daldol",
    "pet_daldol_b",
    "pet_dana",
    "pet_egg",
    "pet_flicky",
    "pet_hamter",
    "pet_majin",
    "pet_merjong",
    "pet_metal",
    "pet_mknux",
    "pet_moonwater",
    "pet_mrpixel",
    "pet_none",
    "pet_patos",
    "pet_perdis",
    "pet_philnux",
    "pet_selfinj",
    "pet_sewers",
    "pet_skull",
    "pet_snic",
    "pet_stor",
    "pet_tits",
    "pet_triz1",
    "pet_triz2",
    "pet_triz3",
    "pet_uncle",
    "pet_whisper",
    "pf_lift",
    "pf_tiles_1",
    "platform",
    "playerescaped",
    "playerhealth",
    "playerhealth_demon",
    "playerhealth_hit",
    "playerhealth_redring",
    "preview_act9_1",
    "preview_act9_2",
    "preview_act9_3",
    "preview_dark_tower_1",
    "preview_dark_tower_2",
    "preview_dark_tower_3",
    "preview_dotdotdot_1",
    "preview_dotdotdot_2",
    "preview_dotdotdot_3",
    "preview_desert_town_1",
    "preview_desert_town_2",
    "preview_desert_town_3",
    "preview_desert_town_background",
    "preview_green_hill_1",
    "preview_green_hill_2",
    "preview_green_hill_3",
    "preview_green_hill_background",
    "preview_haunting_dream_1",
    "preview_haunting_dream_2",
    "preview_haunting_dream_3",
    "preview_haunting_dream_background",
    "preview_angel_island_1",
    "preview_angel_island_2",
    "preview_angel_island_3",
    "preview_angel_island_background",
    "preview_hide_and_seek_2_1",
    "preview_hide_and_seek_2_2",
    "preview_hide_and_seek_2_3",
    "preview_hide_and_seek_2_background",
    "preview_kind_and_fair_1",
    "preview_kind_and_fair_2",
    "preview_kind_and_fair_3",
    "preview_kind_and_fair_background",
    "preview_limp_city_1",
    "preview_limp_city_2",
    "preview_limp_city_3",
    "preview_limp_city_background",
    "preview_marijuna_1",
    "preview_marijuna_2",
    "preview_marijuna_3",
    "preview_marijuna_background",
    "preview_majin_forest_1",
    "preview_majin_forest_2",
    "preview_majin_forest_3",
    "preview_majin_forest_background",
    "preview_nasty_paradise_1",
    "preview_nasty_paradise_2",
    "preview_nasty_paradise_3",
    "preview_nasty_paradise_background",
    "preview_not_perfect_1",
    "preview_not_perfect_2",
    "preview_not_perfect_3",
    "preview_not_perfect_4",
    "preview_not_perfect_background",
    "preview_priceless_freedom_1",
    "preview_priceless_freedom_2",
    "preview_priceless_freedom_3",
    "preview_ravine_mist_1",
    "preview_ravine_mist_2",
    "preview_ravine_mist_background",
    "preview_torture_cave_1",
    "preview_torture_cave_2",
    "preview_torture_cave_3",
    "preview_torture_cave_background",
    "preview_volcano_valley_1",
    "preview_volcano_valley_2",
    "preview_volcano_valley_3",
    "preview_volcano_valley_background",
    "preview_weed_zone_1",
    "preview_weed_zone_2",
    "preview_weed_zone_3",
    "preview_weed_zone_background",
    "preview_you_cant_run_1",
    "preview_you_cant_run_2",
    "preview_you_cant_run_3",
    "preview_you_cant_run_background",
    "preview_fart_zone_1",
    "preview_fart_zone_2",
    "preview_fart_zone_3",
    "ramphelper_right",
    "ravinemist_1",
    "ravinemist2_1",
    "ravinemist_bush",
    "ravinemist_bush_animated",
    "ravinemist_fog1",
    "ravinemist_fog2",
    "ravinemist_shard",
    "ravinemist_sonic",
    "ravinemist_sonic2",
    "ravinemist_sonic3",
    "ravinemist_sonicdead",
    "ravinemist_ui",
    "red",
    "redring",
    "redring_fore",
    "results_bar",
    "results_icons",
    "revival",
    "revival2",
    "ring",
    "ring_sparkle",
    "ring_teleport",
    "ringlose",
    "ringpart",
    "ringpart2",
    "ringpart3",
    "ringpart4",
    "ringspawn",
    "roseheart",
    "sally_attack",
    "sally_balancing",
    "sally_dead",
    "sally_emotion1",
    "sally_emotion2",
    "sally_emotion3",
    "sally_fall",
    "sally_hurt",
    "sally_idle",
    "sally_jump",
    "sally_lookdown",
    "sally_lookup",
    "sally_run",
    "sally_slide",
    "sally_walk",
    "sally_zipline",
    "sallyshield",
    "sallyshield2",
    "screenoverlay",
    "screenoverlay2",
    "selfinsert",
    "selfinsert2",
    "sensor",
    "shard_sparkle",
    "shieldbreak",
    "shieldbreak2",
    "shockparticle",
    "sindicator",
    "sindicator2",
    "sindicator3",
    "smoke",
    "snbway",
    "sota",
    "sota2",
    "sota3",
    "soundemitter",
    "spawnpoint",
    "spike",
    "spike_moving",
    "spike_r",
    "spring_left",
    "spring_right",
    "spring_up",
    "static",
    "status",
    "sudden_death",
    "survivorsescaped",
    "tab_entry",
    "tail_ray_dead",
    "tail_ray_dead1",
    "tail_ray_dead2",
    "tail_ray_dead3",
    "tail_ray_dead4",
    "tails_attack1",
    "tails_balancing",
    "tails_dead",
    "tails_emotion1",
    "tails_emotion2",
    "tails_emotion3",
    "tails_fall",
    "tails_fly",
    "tails_hurt",
    "tails_idle",
    "tails_jump",
    "tails_lookdown",
    "tails_lookup",
    "tails_ray",
    "tails_run",
    "tails_tail1",
    "tails_tail2",
    "tails_walk",
    "tails_zipline",
    "tailscharge",
    "timeover",
    "titlecard1",
    "titlecard2",
    "vote_bars",
    "vv_barrel",
    "vv_barrel2",
    "vv_flava",
    "vv_lava",
    "vv_lavacolumn",
    "vv_lavacolumn2",
    "vv_tiles_1",
    "vv_tiles2_1",
    "vv_tiles3_1",
    "vv_vase",
    "vv_vasepiece",
    "warning",
    "watersplash",
    "weed_arms",
    "weed_chain",
    "weed_chain2",
    "weed_conveyor",
    "weed_conveyor2",
    "weed_conveyor3",
    "weed_latern",
    "weed_light",
    "weed_ghosts",
    "weed_ghosts2",
    "weed_slope_jumpthrough",
    "weed_slope",
    "weed_testiles",
    "weed_tiles_1",
    "weed_tiles2_1",
    "weed_tiles3_1",
    "weed_vignette",
    "white",
    "ycr_slope",
    "ycr_slope2",
    "ycr_tiles_1",
    "ycr_tiles2_1",
    "youkilledeveryone",
    "yspring_up",
    "bluespheres_floor",
    "fire",
    "hs2_left",
    "hs2_platform",
    "hs2_right",
    "hs2_solid",
    "hs2_solid2",
    "hud",
    "lobby_tile3",
    "lobby_tile3_hitbox",
    "lobby_tile4",
    "lobby_tile4_hitbox",
    "ravinemist_bg",
    "ravinemist_misc",
    "ravinemist_trees",
    "tails_spring",
    "goodperson_old",
    "lobby_icon_old",
    "gui_hint_directions",
    "ravinemist_shard_without_glow",
    "act9_tiles_2",
    "aiz_tiles_2",
    "aiz_tiles2_2",
    "am_tiles_2",
    "am_tiles_3",
    "am_tiles_4",
    "am_tiles2_2",
    "am_tiles2_3",
    "am_tiles2_4",
    "am_tiles3_2",
    "am_tiles3_3",
    "am_tiles3_4",
    "darktower_tiles_2",
    "darktower_tiles_3",
    "darktower_tiles_4",
    "deserttown_tiles_2",
    "deserttown_tiles2_2",
    "deserttown_tiles3_2",
    "dot_tiles_2",
    "dot_tiles_3",
    "dot_tiles_4",
    "dot_tiles1_2",
    "dot_tiles1_3",
    "dot_tiles1_4",
    "ghz_tiles_2",
    "ghz_tiles_3",
    "ghz_tiles_4",
    "ghz_tiles2_2",
    "ghz_tiles2_3",
    "ghz_tiles2_4",
    "hd_tiles_2",
    "hd_tiles_3",
    "hd_tiles_4",
    "hd_tiles2_2",
    "hd_tiles2_3",
    "hd_tiles2_4",
    "hs2_tiles_2",
    "hs2_tiles_3",
    "hs2_tiles_4",
    "kindnfair_tiles_2",
    "kindnfair_tiles_3",
    "kindnfair_tiles_4",
    "limpcity_tiles_2",
    "majin_forest_tiles_2",
    "marijuna_tiles_2",
    "marijuna_tiles2_2",
    "nap_tails2_2",
    "nap_tiles_2",
    "notperfect_2",
    "notperfect_3",
    "notperfect_4",
    "notperfect2_2",
    "notperfect2_3",
    "notperfect2_4",
    "notperfect2_p2_2",
    "notperfect2_p2_3",
    "notperfect2_p2_4",
    "notperfect_p2_2",
    "notperfect_p2_3",
    "notperfect_p2_4",
    "pf_tiles_2",
    "pf_tiles_3",
    "pf_tiles_4",
    "ravinemist_2",
    "ravinemist2_2",
    "vv_tiles_2",
    "vv_tiles_3",
    "vv_tiles_4",
    "vv_tiles2_2",
    "vv_tiles2_3",
    "vv_tiles2_4",
    "vv_tiles3_2",
    "vv_tiles3_3",
    "vv_tiles3_4",
    "weed_tiles_2",
    "weed_tiles_3",
    "weed_tiles_4",
    "weed_tiles2_2",
    "weed_tiles2_3",
    "weed_tiles2_4",
    "weed_tiles3_2",
    "weed_tiles3_3",
    "weed_tiles3_4",
    "ycr_tiles_2",
    "ycr_tiles2_2",
    "underwater_noise",
    "preview_fart_zone_background",
    // The cut level of the 12.2022 alpha (map 0) and its vote pictures; after the rest, so no number moves.
    "background_test",
    "lobby_fog",
    "lobby_tile1",
    "lobby_tile2",
    "lobby_tile5",
    "preview_level1_1",
    "preview_level1_2",
    "preview_level1_3",
    // The blank touch buttons and what is drawn on them; after the rest, so no number moves.
    "mobile_smallicons",
    "gui_emotion_misery",
    "gui_emotion_happiness",
];

pub const SOUND_FILES: [&str; 215] = [
    "act9",
    "act9_chase",
    "angelisland",
    "angelisland_chase",
    "bluespheres",
    "darktower",
    "darktower_chase",
    "deserttown",
    "deserttown_chase",
    "dotdotdot",
    "dotdotdot2",
    "dotdotdot_chase",
    "exewin",
    "fartzone",
    "fartzone_chase",
    "hauntdream",
    "hauntdream_chase",
    "hideandseek2",
    "hideandseek2_chase",
    "hill",
    "hill_chase",
    "kindandfair",
    "kindandfair_chase",
    "limpcity",
    "limpcity_chase",
    "lobby",
    "lobby_epic",
    "logo",
    "majinforest",
    "majinforest_chase",
    "marijuna",
    "marijuna_chase",
    "menu",
    "mindfuck",
    "minigame",
    "nastyparadise",
    "nastyparadise_chase",
    "notperfect",
    "notperfect_chase",
    "pricelessfreedom",
    "pricelessfreedom_chase",
    "ravimemist",
    "ravimemist_chase",
    "survwin",
    "timeover",
    "torturecave",
    "torturecave_chase",
    "volcanovalley",
    "volcanovalley_chase",
    "waiting",
    "weedzone",
    "weedzone_chase",
    "youcantrun",
    "youcantrun_chase",
    "achivement",
    "acid",
    "blackring",
    "blackring_ball",
    "boohoo",
    "boom",
    "bos",
    "break",
    "buble",
    "chaos_attack",
    "chaos_dash",
    "chaos_kill",
    "chaos_kill2",
    "chaos_kill3",
    "chaos_kill4",
    "chaos_kill5",
    "chaos_kill6",
    "chaos_kill7",
    "chaos_kill8",
    "chaos_land",
    "chaos_laugh",
    "chaos_pizza",
    "chaos_stun",
    "chaos_sturn",
    "chaos_taunt",
    "chaos_taunt2",
    "chaos_vineboom",
    "clock",
    "creamdash",
    "creamring",
    "dash",
    "dead",
    "demonization",
    "destiny",
    "door",
    "dummy",
    "dummy2",
    "dummy3",
    "echain",
    "echain_prepare",
    "egg_djump",
    "egg_shield",
    "egg_tracker",
    "egg_tracker_activate",
    "electroshock",
    "exe_appear",
    "exe_appear2",
    "exe_appear3",
    "exe_invisenter",
    "exe_invisenter2",
    "exe_kill1",
    "exe_kill2",
    "exe_kill3",
    "exe_kill4",
    "exe_laugh",
    "exe_ringshutter",
    "exe_stun",
    "exe_stun2",
    "exe_taunt",
    "exe_taunt1",
    "exe_taunt2",
    "exe_wins",
    "exeller_clone",
    "exeller_cloneline",
    "exeller_kill1",
    "exeller_kill2",
    "exeller_kill3",
    "exeller_kill4",
    "exeller_kill5",
    "exeller_kill6",
    "exeller_kill7",
    "exeller_laugh",
    "exeller_stun",
    "exeller_taunt1",
    "exeller_taunt2",
    "exeller_taunt3",
    "exetior_kill1",
    "exetior_kill2",
    "exetior_kill3",
    "exetior_kill4",
    "exetior_kill5",
    "exetior_kill6",
    "exetior_laugh",
    "exetior_ring1",
    "exetior_ring2",
    "exetior_ring3",
    "exetior_ring4",
    "exetior_shockwave",
    "exetior_stomp",
    "exetior_stompland",
    "exetior_stun",
    "exetior_taunt1",
    "exetior_taunt2",
    "exetior_taunt3",
    "expweak",
    "fart",
    "fnac",
    "heal",
    "hurt",
    "ice_break",
    "ice_spawn",
    "judger",
    "jump",
    "lava",
    "lavaappear",
    "lavahit",
    "lift",
    "menu_press",
    "menu_select",
    "mermer",
    "message",
    "minibal",
    "minidestroy",
    "minidie",
    "miniextra",
    "minispin",
    "ministart",
    "movingspike",
    "nono",
    "npteleport",
    "rain",
    "ready",
    "redring",
    "ring",
    "ringabsorb",
    "ringlose",
    "roar",
    "sally_shield",
    "sally_shieldbreak",
    "sally_slide",
    "shard",
    "stalactite_notice",
    "slime",
    "smoke",
    "snap",
    "snowball_break",
    "snowball_roll",
    "spike",
    "spin",
    "spring",
    "spring_reverb",
    "suddendeath",
    "survivor_win",
    "tails_charge",
    "tails_fly",
    "tails_hit",
    "tails_shoot",
    "tailsball",
    "tailsball_chase",
    "tailsball_jumpscare",
    "tailsball_jumpscare2",
    "teleport",
    "thunder",
    "vasebreak",
    "waterdrops",
    "watersplash",
    "wdarms",
    "wdlamp",
    // Cut content brought back (CutContent/README.md); after the rest, so no number moves.
    "chase",
    "level",
    "pns",
];

pub mod sprite {
    use super::SpriteId;
    pub const BACKGROUND_ACT9: SpriteId = SpriteId(0);
    pub const BACKGROUND_DARKTOWER: SpriteId = SpriteId(16);
    pub const BACKGROUND_DOTDOTDOT: SpriteId = SpriteId(17);
    pub const BACKGROUND_DOTDOTDOT2: SpriteId = SpriteId(18);
    pub const BACKGROUND_DOTDOTDOT3: SpriteId = SpriteId(19);
    /// Our own noise the heat haze of Angel Island and Volcano Valley shimmers with.
    pub const SPR_HEATHAZE_NOISE: SpriteId = SpriteId(36);
    pub const BACKGROUND_NOTPERF: SpriteId = SpriteId(59);
    pub const BACKGROUND_NOTPERF2: SpriteId = SpriteId(60);
    pub const BACKGROUND_NOTPERF3: SpriteId = SpriteId(61);
    pub const BACKGROUND_PF: SpriteId = SpriteId(62);
    pub const SPD_HD_COLLIISON: SpriteId = SpriteId(80);
    pub const SPR_ABYSS: SpriteId = SpriteId(81);
    pub const SPR_ABYSS_TARGET: SpriteId = SpriteId(82);
    pub const SPR_ACHIVARROWS: SpriteId = SpriteId(83);
    pub const SPR_ACHIVEMENTBOX: SpriteId = SpriteId(84);
    pub const SPR_ACHIVEMENTS: SpriteId = SpriteId(85);
    pub const SPR_ACT9_DEAD: SpriteId = SpriteId(87);
    pub const SPR_ACT9_WALL: SpriteId = SpriteId(89);
    pub const SPR_AIZ_DECOR: SpriteId = SpriteId(90);
    pub const SPR_AIZ_DECOR2: SpriteId = SpriteId(91);
    pub const SPR_AIZ_ZIPLINE: SpriteId = SpriteId(97);
    pub const SPR_AM_CLOUD: SpriteId = SpriteId(99);
    pub const SPR_AM_COLLISION: SpriteId = SpriteId(100);
    pub const SPR_AM_FACE: SpriteId = SpriteId(101);
    pub const SPR_AM_OOH: SpriteId = SpriteId(102);
    pub const SPR_AMY_ATTACK1: SpriteId = SpriteId(106);
    pub const SPR_AMY_ATTACK2: SpriteId = SpriteId(107);
    pub const SPR_AMY_BALANCING: SpriteId = SpriteId(108);
    pub const SPR_AMY_DEAD: SpriteId = SpriteId(109);
    pub const SPR_AMY_EMOTION1: SpriteId = SpriteId(110);
    pub const SPR_AMY_EMOTION2: SpriteId = SpriteId(111);
    pub const SPR_AMY_EMOTION3: SpriteId = SpriteId(112);
    pub const SPR_AMY_FALL: SpriteId = SpriteId(113);
    pub const SPR_AMY_HURT: SpriteId = SpriteId(114);
    pub const SPR_AMY_IDLE: SpriteId = SpriteId(115);
    pub const SPR_AMY_JUMP: SpriteId = SpriteId(116);
    pub const SPR_AMY_LOOKDOWN: SpriteId = SpriteId(117);
    pub const SPR_AMY_LOOKUP: SpriteId = SpriteId(118);
    pub const SPR_AMY_RUN: SpriteId = SpriteId(119);
    pub const SPR_AMY_WALK: SpriteId = SpriteId(120);
    pub const SPR_AMY_ZIPLINE: SpriteId = SpriteId(121);
    pub const SPR_ATTACKGUI: SpriteId = SpriteId(122);
    pub const SPR_BACKGROUND_TEST: SpriteId = SpriteId(984);
    pub const SPR_BADCONNECTION: SpriteId = SpriteId(123);
    pub const SPR_BIGRING: SpriteId = SpriteId(124);
    pub const SPR_BIGRING_READY: SpriteId = SpriteId(125);
    pub const SPR_BLACK: SpriteId = SpriteId(126);
    pub const SPR_BLACKRING: SpriteId = SpriteId(127);
    pub const SPR_BLOCK: SpriteId = SpriteId(131);
    pub const SPR_BLOCK2: SpriteId = SpriteId(132);
    pub const SPR_BLOOD1: SpriteId = SpriteId(133);
    pub const SPR_BLOOD2: SpriteId = SpriteId(134);
    pub const SPR_BLOOD3: SpriteId = SpriteId(135);
    pub const SPR_BLUESPHERES_SPHERE: SpriteId = SpriteId(136);
    pub const SPR_BOOM: SpriteId = SpriteId(137);
    pub const SPR_BSPRING_LEFT: SpriteId = SpriteId(138);
    pub const SPR_BSPRING_RIGHT: SpriteId = SpriteId(139);
    pub const SPR_BSPRING_UP: SpriteId = SpriteId(140);
    pub const SPR_CHAOS_ATTACK1: SpriteId = SpriteId(141);
    pub const SPR_CHAOS_ATTACK2: SpriteId = SpriteId(142);
    pub const SPR_CHAOS_BALANCING: SpriteId = SpriteId(143);
    pub const SPR_CHAOS_EMOTION1: SpriteId = SpriteId(144);
    pub const SPR_CHAOS_EMOTION2: SpriteId = SpriteId(145);
    pub const SPR_CHAOS_EMOTION3: SpriteId = SpriteId(146);
    pub const SPR_CHAOS_FALL: SpriteId = SpriteId(147);
    pub const SPR_CHAOS_HURT: SpriteId = SpriteId(148);
    pub const SPR_CHAOS_IDLE: SpriteId = SpriteId(149);
    pub const SPR_CHAOS_JUMP: SpriteId = SpriteId(150);
    pub const SPR_CHAOS_LIQUID: SpriteId = SpriteId(151);
    pub const SPR_CHAOS_LOOKDOWN: SpriteId = SpriteId(152);
    pub const SPR_CHAOS_LOOKUP: SpriteId = SpriteId(153);
    pub const SPR_CHAOS_LOST: SpriteId = SpriteId(154);
    pub const SPR_CHAOS_LOST2: SpriteId = SpriteId(155);
    pub const SPR_CHAOS_RUN: SpriteId = SpriteId(156);
    pub const SPR_CHAOS_SFALL: SpriteId = SpriteId(157);
    pub const SPR_CHAOS_SIDLE: SpriteId = SpriteId(158);
    pub const SPR_CHAOS_SJUMP: SpriteId = SpriteId(159);
    pub const SPR_CHAOS_STRANSFORM: SpriteId = SpriteId(160);
    pub const SPR_CHAOS_STRANSFORMAIR: SpriteId = SpriteId(161);
    pub const SPR_CHAOS_STUCK: SpriteId = SpriteId(162);
    pub const SPR_CHAOS_STUCK2: SpriteId = SpriteId(163);
    pub const SPR_CHAOS_STUN: SpriteId = SpriteId(164);
    pub const SPR_CHAOS_SWALK: SpriteId = SpriteId(165);
    pub const SPR_CHAOS_WALK: SpriteId = SpriteId(166);
    pub const SPR_CHAOS_WON: SpriteId = SpriteId(167);
    pub const SPR_CHAOS_ZIPLINE: SpriteId = SpriteId(168);
    pub const SPR_CHAR_BARS: SpriteId = SpriteId(169);
    pub const SPR_CHAR_INFO: SpriteId = SpriteId(170);
    pub const SPR_CLOCK: SpriteId = SpriteId(171);
    pub const SPR_CLOCKHAND: SpriteId = SpriteId(172);
    pub const SPR_CORPSE_AMY: SpriteId = SpriteId(173);
    pub const SPR_CORPSE_CREAM: SpriteId = SpriteId(174);
    pub const SPR_CORPSE_EGGMAN: SpriteId = SpriteId(175);
    pub const SPR_CORPSE_KNUX: SpriteId = SpriteId(176);
    pub const SPR_CORPSE_TAILS: SpriteId = SpriteId(177);
    pub const SPR_COUNTER: SpriteId = SpriteId(178);
    pub const SPR_COUNTER2: SpriteId = SpriteId(179);
    pub const SPR_CREAM_BALANCING: SpriteId = SpriteId(180);
    pub const SPR_CREAM_DEAD: SpriteId = SpriteId(181);
    pub const SPR_CREAM_EMOTION1: SpriteId = SpriteId(182);
    pub const SPR_CREAM_EMOTION2: SpriteId = SpriteId(183);
    pub const SPR_CREAM_EMOTION3: SpriteId = SpriteId(184);
    pub const SPR_CREAM_FALL: SpriteId = SpriteId(185);
    pub const SPR_CREAM_FLY: SpriteId = SpriteId(186);
    pub const SPR_CREAM_HURT: SpriteId = SpriteId(187);
    pub const SPR_CREAM_IDLE: SpriteId = SpriteId(188);
    pub const SPR_CREAM_JUMP: SpriteId = SpriteId(189);
    pub const SPR_CREAM_LOOKDOWN: SpriteId = SpriteId(190);
    pub const SPR_CREAM_LOOKUP: SpriteId = SpriteId(191);
    pub const SPR_CREAM_RUN: SpriteId = SpriteId(192);
    pub const SPR_CREAM_SRINGS: SpriteId = SpriteId(193);
    pub const SPR_CREAM_WALK: SpriteId = SpriteId(194);
    pub const SPR_CREAM_ZIPLINE: SpriteId = SpriteId(195);
    pub const SPR_DARKTOWER_BALL: SpriteId = SpriteId(196);
    pub const SPR_DARKTOWER_FOG: SpriteId = SpriteId(197);
    pub const SPR_DARKTOWER_JUMPSCARE: SpriteId = SpriteId(198);
    pub const SPR_DARKTOWER_STALACTITE: SpriteId = SpriteId(199);
    pub const SPR_DARKTOWER_STALACTITE1: SpriteId = SpriteId(200);
    pub const SPR_DARKTOWER_STALACTITE2: SpriteId = SpriteId(201);
    pub const SPR_DARKTOWER_TAILSOL: SpriteId = SpriteId(202);
    pub const SPR_DEATHTP: SpriteId = SpriteId(204);
    pub const SPR_DEATHTP_POINT: SpriteId = SpriteId(205);
    pub const SPR_DESERTTOWN_HIDE: SpriteId = SpriteId(207);
    pub const SPR_DESERTTOWN_HIDE2: SpriteId = SpriteId(208);
    pub const SPR_DESERTTOWN_MISC: SpriteId = SpriteId(209);
    pub const SPR_DESERTTOWN_MISCTILES: SpriteId = SpriteId(210);
    pub const SPR_DESERTTOWN_TILES_1: SpriteId = SpriteId(211);
    pub const SPR_DOT_EGGSTATUE: SpriteId = SpriteId(214);
    pub const SPR_DUST: SpriteId = SpriteId(220);
    pub const SPR_EAMY_ATTACK1: SpriteId = SpriteId(221);
    pub const SPR_EAMY_ATTACK2: SpriteId = SpriteId(222);
    pub const SPR_EAMY_BALANCING: SpriteId = SpriteId(223);
    pub const SPR_EAMY_EMOTION1: SpriteId = SpriteId(224);
    pub const SPR_EAMY_EMOTION2: SpriteId = SpriteId(225);
    pub const SPR_EAMY_EMOTION3: SpriteId = SpriteId(226);
    pub const SPR_EAMY_FALL: SpriteId = SpriteId(227);
    pub const SPR_EAMY_HURT: SpriteId = SpriteId(228);
    pub const SPR_EAMY_IDLE: SpriteId = SpriteId(229);
    pub const SPR_EAMY_JUMP: SpriteId = SpriteId(230);
    pub const SPR_EAMY_LOOKDOWN: SpriteId = SpriteId(231);
    pub const SPR_EAMY_LOOKUP: SpriteId = SpriteId(232);
    pub const SPR_EAMY_RUN: SpriteId = SpriteId(233);
    pub const SPR_EAMY_STUN: SpriteId = SpriteId(234);
    pub const SPR_EAMY_WALK: SpriteId = SpriteId(235);
    pub const SPR_EAMY_ZIPLINE: SpriteId = SpriteId(236);
    pub const SPR_ECREAM_BALANCING: SpriteId = SpriteId(237);
    pub const SPR_ECREAM_EMOTION1: SpriteId = SpriteId(238);
    pub const SPR_ECREAM_EMOTION2: SpriteId = SpriteId(239);
    pub const SPR_ECREAM_EMOTION3: SpriteId = SpriteId(240);
    pub const SPR_ECREAM_FALL: SpriteId = SpriteId(241);
    pub const SPR_ECREAM_FLY: SpriteId = SpriteId(242);
    pub const SPR_ECREAM_HURT: SpriteId = SpriteId(243);
    pub const SPR_ECREAM_IDLE: SpriteId = SpriteId(244);
    pub const SPR_ECREAM_JUMP: SpriteId = SpriteId(245);
    pub const SPR_ECREAM_LOOKDOWN: SpriteId = SpriteId(246);
    pub const SPR_ECREAM_LOOKUP: SpriteId = SpriteId(247);
    pub const SPR_ECREAM_RUN: SpriteId = SpriteId(248);
    pub const SPR_ECREAM_SRINGS: SpriteId = SpriteId(249);
    pub const SPR_ECREAM_STUN: SpriteId = SpriteId(250);
    pub const SPR_ECREAM_WALK: SpriteId = SpriteId(251);
    pub const SPR_ECREAM_ZIPLINE: SpriteId = SpriteId(252);
    pub const SPR_EEGG_BALANCING: SpriteId = SpriteId(253);
    pub const SPR_EEGG_DJUMP: SpriteId = SpriteId(254);
    pub const SPR_EEGG_EMOTION1: SpriteId = SpriteId(255);
    pub const SPR_EEGG_EMOTION2: SpriteId = SpriteId(256);
    pub const SPR_EEGG_EMOTION3: SpriteId = SpriteId(257);
    pub const SPR_EEGG_FALL: SpriteId = SpriteId(258);
    pub const SPR_EEGG_HURT: SpriteId = SpriteId(259);
    pub const SPR_EEGG_IDLE: SpriteId = SpriteId(260);
    pub const SPR_EEGG_JUMP: SpriteId = SpriteId(261);
    pub const SPR_EEGG_LOOKDOWN: SpriteId = SpriteId(262);
    pub const SPR_EEGG_LOOKUP: SpriteId = SpriteId(263);
    pub const SPR_EEGG_RUN: SpriteId = SpriteId(264);
    pub const SPR_EEGG_STUN: SpriteId = SpriteId(265);
    pub const SPR_EEGG_WALK: SpriteId = SpriteId(266);
    pub const SPR_EEGG_ZIPLINE: SpriteId = SpriteId(267);
    pub const SPR_EGG_BALANCING: SpriteId = SpriteId(268);
    pub const SPR_EGG_DEAD: SpriteId = SpriteId(269);
    pub const SPR_EGG_DJUMP: SpriteId = SpriteId(270);
    pub const SPR_EGG_EMOTION1: SpriteId = SpriteId(271);
    pub const SPR_EGG_EMOTION2: SpriteId = SpriteId(272);
    pub const SPR_EGG_EMOTION3: SpriteId = SpriteId(273);
    pub const SPR_EGG_FALL: SpriteId = SpriteId(274);
    pub const SPR_EGG_HURT: SpriteId = SpriteId(275);
    pub const SPR_EGG_IDLE: SpriteId = SpriteId(276);
    pub const SPR_EGG_JUMP: SpriteId = SpriteId(277);
    pub const SPR_EGG_LOOKDOWN: SpriteId = SpriteId(278);
    pub const SPR_EGG_LOOKUP: SpriteId = SpriteId(279);
    pub const SPR_EGG_RUN: SpriteId = SpriteId(280);
    pub const SPR_EGG_WALK: SpriteId = SpriteId(281);
    pub const SPR_EGG_ZIPLINE: SpriteId = SpriteId(282);
    pub const SPR_EGGPACK: SpriteId = SpriteId(283);
    pub const SPR_EGGTRACK: SpriteId = SpriteId(284);
    pub const SPR_EGGTRACK_DESTROY: SpriteId = SpriteId(285);
    pub const SPR_EKNUX_ATTACK1: SpriteId = SpriteId(286);
    pub const SPR_EKNUX_ATTACK2: SpriteId = SpriteId(287);
    pub const SPR_EKNUX_BALANCING: SpriteId = SpriteId(288);
    pub const SPR_EKNUX_EMOTION1: SpriteId = SpriteId(289);
    pub const SPR_EKNUX_EMOTION2: SpriteId = SpriteId(290);
    pub const SPR_EKNUX_EMOTION3: SpriteId = SpriteId(291);
    pub const SPR_EKNUX_FALL: SpriteId = SpriteId(292);
    pub const SPR_EKNUX_GLIDE: SpriteId = SpriteId(293);
    pub const SPR_EKNUX_HURT: SpriteId = SpriteId(294);
    pub const SPR_EKNUX_IDLE: SpriteId = SpriteId(295);
    pub const SPR_EKNUX_JUMP: SpriteId = SpriteId(296);
    pub const SPR_EKNUX_LOOKDOWN: SpriteId = SpriteId(297);
    pub const SPR_EKNUX_LOOKUP: SpriteId = SpriteId(298);
    pub const SPR_EKNUX_RUN: SpriteId = SpriteId(299);
    pub const SPR_EKNUX_STUCK: SpriteId = SpriteId(300);
    pub const SPR_EKNUX_STUN: SpriteId = SpriteId(301);
    pub const SPR_EKNUX_WALK: SpriteId = SpriteId(302);
    pub const SPR_EKNUX_ZIPLINE: SpriteId = SpriteId(303);
    pub const SPR_ELECTROSHIELD: SpriteId = SpriteId(304);
    pub const SPR_EROSEHEART: SpriteId = SpriteId(305);
    pub const SPR_ESALLY_ATTACK: SpriteId = SpriteId(306);
    pub const SPR_ESALLY_BALANCING: SpriteId = SpriteId(307);
    pub const SPR_ESALLY_EMOTION1: SpriteId = SpriteId(308);
    pub const SPR_ESALLY_EMOTION2: SpriteId = SpriteId(309);
    pub const SPR_ESALLY_EMOTION3: SpriteId = SpriteId(310);
    pub const SPR_ESALLY_FALL: SpriteId = SpriteId(311);
    pub const SPR_ESALLY_HURT: SpriteId = SpriteId(312);
    pub const SPR_ESALLY_IDLE: SpriteId = SpriteId(313);
    pub const SPR_ESALLY_JUMP: SpriteId = SpriteId(314);
    pub const SPR_ESALLY_LOOKDOWN: SpriteId = SpriteId(315);
    pub const SPR_ESALLY_LOOKUP: SpriteId = SpriteId(316);
    pub const SPR_ESALLY_RUN: SpriteId = SpriteId(317);
    pub const SPR_ESALLY_SLIDE: SpriteId = SpriteId(318);
    pub const SPR_ESALLY_STUN: SpriteId = SpriteId(319);
    pub const SPR_ESALLY_WALK: SpriteId = SpriteId(320);
    pub const SPR_ESALLY_ZIPLINE: SpriteId = SpriteId(321);
    pub const SPR_ETAILS_ATTACK1: SpriteId = SpriteId(322);
    pub const SPR_ETAILS_BALANCING: SpriteId = SpriteId(323);
    pub const SPR_ETAILS_EMOTION1: SpriteId = SpriteId(324);
    pub const SPR_ETAILS_EMOTION2: SpriteId = SpriteId(325);
    pub const SPR_ETAILS_EMOTION3: SpriteId = SpriteId(326);
    pub const SPR_ETAILS_FALL: SpriteId = SpriteId(327);
    pub const SPR_ETAILS_FLY: SpriteId = SpriteId(328);
    pub const SPR_ETAILS_HURT: SpriteId = SpriteId(329);
    pub const SPR_ETAILS_IDLE: SpriteId = SpriteId(330);
    pub const SPR_ETAILS_JUMP: SpriteId = SpriteId(331);
    pub const SPR_ETAILS_LOOKDOWN: SpriteId = SpriteId(332);
    pub const SPR_ETAILS_LOOKUP: SpriteId = SpriteId(333);
    pub const SPR_ETAILS_RUN: SpriteId = SpriteId(334);
    pub const SPR_ETAILS_STUN: SpriteId = SpriteId(335);
    pub const SPR_ETAILS_TAIL1: SpriteId = SpriteId(336);
    pub const SPR_ETAILS_TAIL2: SpriteId = SpriteId(337);
    pub const SPR_ETAILS_WALK: SpriteId = SpriteId(338);
    pub const SPR_ETAILS_ZIPLINE: SpriteId = SpriteId(339);
    pub const SPR_EXE_ATTACK1: SpriteId = SpriteId(340);
    pub const SPR_EXE_ATTACK2: SpriteId = SpriteId(341);
    pub const SPR_EXE_BALANCING: SpriteId = SpriteId(342);
    pub const SPR_EXE_EMOTION: SpriteId = SpriteId(343);
    pub const SPR_EXE_EMOTION2: SpriteId = SpriteId(344);
    pub const SPR_EXE_EMOTION3: SpriteId = SpriteId(345);
    pub const SPR_EXE_FALL: SpriteId = SpriteId(346);
    pub const SPR_EXE_HURT: SpriteId = SpriteId(347);
    pub const SPR_EXE_IDLE: SpriteId = SpriteId(348);
    pub const SPR_EXE_INVIS_BALANCING: SpriteId = SpriteId(349);
    pub const SPR_EXE_INVIS_EMOTION: SpriteId = SpriteId(350);
    pub const SPR_EXE_INVIS_FALL: SpriteId = SpriteId(351);
    pub const SPR_EXE_INVIS_IDLE: SpriteId = SpriteId(353);
    pub const SPR_EXE_INVIS_JUMP: SpriteId = SpriteId(354);
    pub const SPR_EXE_INVIS_LOOKDOWN: SpriteId = SpriteId(355);
    pub const SPR_EXE_INVIS_LOOKUP: SpriteId = SpriteId(356);
    pub const SPR_EXE_INVIS_RUN: SpriteId = SpriteId(357);
    pub const SPR_EXE_INVIS_WALK: SpriteId = SpriteId(358);
    pub const SPR_EXE_INVIS_ZIPLINE: SpriteId = SpriteId(359);
    pub const SPR_EXE_JUMP: SpriteId = SpriteId(360);
    pub const SPR_EXE_LOOKDOWN: SpriteId = SpriteId(361);
    pub const SPR_EXE_LOOKUP: SpriteId = SpriteId(362);
    pub const SPR_EXE_LOST: SpriteId = SpriteId(363);
    pub const SPR_EXE_RUN: SpriteId = SpriteId(364);
    pub const SPR_EXE_SHOCKED: SpriteId = SpriteId(365);
    pub const SPR_EXE_WALK: SpriteId = SpriteId(366);
    pub const SPR_EXE_ZIPLINE: SpriteId = SpriteId(368);
    pub const SPR_EXELLER_ATTACK1: SpriteId = SpriteId(369);
    pub const SPR_EXELLER_ATTACK2: SpriteId = SpriteId(370);
    pub const SPR_EXELLER_BALANCING: SpriteId = SpriteId(371);
    pub const SPR_EXELLER_CLONE: SpriteId = SpriteId(372);
    pub const SPR_EXELLER_CLONE2: SpriteId = SpriteId(373);
    pub const SPR_EXELLER_CLONEARROW: SpriteId = SpriteId(374);
    pub const SPR_EXELLER_CLONETRACKER: SpriteId = SpriteId(375);
    pub const SPR_EXELLER_CLONETRACKER2: SpriteId = SpriteId(376);
    pub const SPR_EXELLER_EMOTION: SpriteId = SpriteId(377);
    pub const SPR_EXELLER_EMOTION2: SpriteId = SpriteId(378);
    pub const SPR_EXELLER_EMOTION3: SpriteId = SpriteId(379);
    pub const SPR_EXELLER_FALL: SpriteId = SpriteId(380);
    pub const SPR_EXELLER_HURT: SpriteId = SpriteId(381);
    pub const SPR_EXELLER_IDLE: SpriteId = SpriteId(382);
    pub const SPR_EXELLER_JUMP: SpriteId = SpriteId(383);
    pub const SPR_EXELLER_LOOKDOWN: SpriteId = SpriteId(384);
    pub const SPR_EXELLER_LOOKUP: SpriteId = SpriteId(385);
    pub const SPR_EXELLER_LOST: SpriteId = SpriteId(386);
    pub const SPR_EXELLER_LOST2: SpriteId = SpriteId(387);
    pub const SPR_EXELLER_RUN: SpriteId = SpriteId(388);
    pub const SPR_EXELLER_SHOCKED: SpriteId = SpriteId(389);
    pub const SPR_EXELLER_WALK: SpriteId = SpriteId(390);
    pub const SPR_EXELLER_WON: SpriteId = SpriteId(391);
    pub const SPR_EXELLER_ZIPLINE: SpriteId = SpriteId(392);
    pub const SPR_EXESPAWN: SpriteId = SpriteId(393);
    pub const SPR_EXETIOR_ATTACK1: SpriteId = SpriteId(394);
    pub const SPR_EXETIOR_ATTACK2: SpriteId = SpriteId(395);
    pub const SPR_EXETIOR_BALANCING: SpriteId = SpriteId(396);
    pub const SPR_EXETIOR_EMOTION: SpriteId = SpriteId(397);
    pub const SPR_EXETIOR_EMOTION2: SpriteId = SpriteId(398);
    pub const SPR_EXETIOR_EMOTION3: SpriteId = SpriteId(399);
    pub const SPR_EXETIOR_FALL: SpriteId = SpriteId(400);
    pub const SPR_EXETIOR_HURT: SpriteId = SpriteId(401);
    pub const SPR_EXETIOR_IDLE: SpriteId = SpriteId(402);
    pub const SPR_EXETIOR_JUMP: SpriteId = SpriteId(403);
    pub const SPR_EXETIOR_LOOKDOWN: SpriteId = SpriteId(404);
    pub const SPR_EXETIOR_LOOKUP: SpriteId = SpriteId(405);
    pub const SPR_EXETIOR_LOST: SpriteId = SpriteId(406);
    pub const SPR_EXETIOR_RUN: SpriteId = SpriteId(407);
    pub const SPR_EXETIOR_SHOCKED: SpriteId = SpriteId(408);
    pub const SPR_EXETIOR_STOMP: SpriteId = SpriteId(409);
    pub const SPR_EXETIOR_STOMPBALLS: SpriteId = SpriteId(410);
    pub const SPR_EXETIOR_STOMPLAND: SpriteId = SpriteId(411);
    pub const SPR_EXETIOR_WALK: SpriteId = SpriteId(412);
    pub const SPR_EXETIOR_WON: SpriteId = SpriteId(413);
    pub const SPR_EXETIOR_ZIPLINE: SpriteId = SpriteId(414);
    pub const SPR_EXPLOSION: SpriteId = SpriteId(415);
    pub const SPR_FART_DUMMY: SpriteId = SpriteId(416);
    pub const SPR_FARTZONE: SpriteId = SpriteId(417);
    pub const SPR_FROZEN: SpriteId = SpriteId(418);
    pub const SPR_GAMEOVER: SpriteId = SpriteId(419);
    pub const SPR_GHZ_ELECTROSHOCK: SpriteId = SpriteId(420);
    pub const SPR_GHZ_RAIN: SpriteId = SpriteId(422);
    pub const SPR_GHZ_SLOPE1: SpriteId = SpriteId(423);
    pub const SPR_GHZ_SLOPE2: SpriteId = SpriteId(424);
    pub const SPR_GHZ_SLOPE3: SpriteId = SpriteId(425);
    pub const SPR_GHZ_SLOPE4: SpriteId = SpriteId(426);
    pub const SPR_GHZ_SLOPE5: SpriteId = SpriteId(427);
    pub const SPR_GHZ_WAVE: SpriteId = SpriteId(432);
    pub const SPR_GOODPERSON: SpriteId = SpriteId(433);
    pub const SPR_GREEN: SpriteId = SpriteId(434);
    pub const SPR_GREEN293: SpriteId = SpriteId(435);
    pub const SPR_GUI_AMYATTACK: SpriteId = SpriteId(436);
    pub const SPR_GUI_AMYHJUMP: SpriteId = SpriteId(437);
    pub const SPR_GUI_CHAOSATTACK: SpriteId = SpriteId(438);
    pub const SPR_GUI_CHAOSSLIME: SpriteId = SpriteId(439);
    pub const SPR_GUI_CHAOSWALLDASH: SpriteId = SpriteId(440);
    pub const SPR_GUI_CREAMDASH: SpriteId = SpriteId(441);
    pub const SPR_GUI_CREAMFLY: SpriteId = SpriteId(442);
    pub const SPR_GUI_CREAMRINGS: SpriteId = SpriteId(443);
    pub const SPR_GUI_EGGDJUMP: SpriteId = SpriteId(444);
    pub const SPR_GUI_EGGSHIELD: SpriteId = SpriteId(445);
    pub const SPR_GUI_EGGTRACK: SpriteId = SpriteId(446);
    pub const SPR_GUI_EMOTION_ANGER: SpriteId = SpriteId(447);
    pub const SPR_GUI_EMOTION_MISERY: SpriteId = SpriteId(993);
    pub const SPR_GUI_EMOTION_HAPPINESS: SpriteId = SpriteId(994);
    pub const SPR_GUI_EXEATTACK: SpriteId = SpriteId(448);
    pub const SPR_GUI_EXEFREEJUMP: SpriteId = SpriteId(449);
    pub const SPR_GUI_EXEINVISABILITY: SpriteId = SpriteId(450);
    pub const SPR_GUI_EXELLERCLONE: SpriteId = SpriteId(451);
    pub const SPR_GUI_EXELLERCLONE2: SpriteId = SpriteId(452);
    pub const SPR_GUI_EXETIORATTACK: SpriteId = SpriteId(453);
    pub const SPR_GUI_EXETIORRING: SpriteId = SpriteId(454);
    pub const SPR_GUI_KNUXATTACK: SpriteId = SpriteId(455);
    pub const SPR_GUI_KNUXGLIDE: SpriteId = SpriteId(456);
    pub const SPR_GUI_SALLYATTACK: SpriteId = SpriteId(457);
    pub const SPR_GUI_SALLYSHIELD: SpriteId = SpriteId(458);
    pub const SPR_GUI_TAILSATTACK: SpriteId = SpriteId(459);
    pub const SPR_GUI_TAILSFLY: SpriteId = SpriteId(460);
    pub const SPR_HD_CRYSTAL: SpriteId = SpriteId(461);
    pub const SPR_HD_DOOR: SpriteId = SpriteId(463);
    pub const SPR_HD_DOOR2: SpriteId = SpriteId(464);
    pub const SPR_HD_HIDE: SpriteId = SpriteId(465);
    pub const SPR_HD_SPRING: SpriteId = SpriteId(466);
    pub const SPR_HEAL: SpriteId = SpriteId(469);
    pub const SPR_HEAL_PART: SpriteId = SpriteId(470);
    pub const SPR_HIDEGUI: SpriteId = SpriteId(471);
    pub const SPR_HP: SpriteId = SpriteId(472);
    pub const SPR_HP_RR: SpriteId = SpriteId(473);
    pub const SPR_HS2_BOX: SpriteId = SpriteId(474);
    pub const SPR_HS2_SLOPE1: SpriteId = SpriteId(475);
    pub const SPR_HS2_SLOPE2: SpriteId = SpriteId(476);
    pub const SPR_INDICATOR: SpriteId = SpriteId(478);
    pub const SPR_INDICATOR2: SpriteId = SpriteId(479);
    pub const SPR_INDICATOR3: SpriteId = SpriteId(480);
    pub const SPR_KINDNFAIR_SLOPE1: SpriteId = SpriteId(481);
    pub const SPR_KINDNFAIR_SLOPE2: SpriteId = SpriteId(482);
    pub const SPR_KINDNFAIR_SPEEDBOX: SpriteId = SpriteId(483);
    pub const SPR_KNUX_ATTACK1: SpriteId = SpriteId(485);
    pub const SPR_KNUX_ATTACK2: SpriteId = SpriteId(486);
    pub const SPR_KNUX_BALANCING: SpriteId = SpriteId(487);
    pub const SPR_KNUX_DEAD: SpriteId = SpriteId(488);
    pub const SPR_KNUX_EMOTION1: SpriteId = SpriteId(489);
    pub const SPR_KNUX_EMOTION2: SpriteId = SpriteId(490);
    pub const SPR_KNUX_EMOTION3: SpriteId = SpriteId(491);
    pub const SPR_KNUX_FALL: SpriteId = SpriteId(492);
    pub const SPR_KNUX_GLIDE: SpriteId = SpriteId(493);
    pub const SPR_KNUX_HURT: SpriteId = SpriteId(494);
    pub const SPR_KNUX_IDLE: SpriteId = SpriteId(495);
    pub const SPR_KNUX_JUMP: SpriteId = SpriteId(496);
    pub const SPR_KNUX_LOOKDOWN: SpriteId = SpriteId(497);
    pub const SPR_KNUX_LOOKUP: SpriteId = SpriteId(498);
    pub const SPR_KNUX_MARIJUNA: SpriteId = SpriteId(499);
    pub const SPR_KNUX_RUN: SpriteId = SpriteId(500);
    pub const SPR_KNUX_STUCK: SpriteId = SpriteId(501);
    pub const SPR_KNUX_WALK: SpriteId = SpriteId(502);
    pub const SPR_KNUX_ZIPLINE: SpriteId = SpriteId(503);
    pub const SPR_LIMPCITY_CHAIN1: SpriteId = SpriteId(504);
    pub const SPR_LIMPCITY_CHAIN2: SpriteId = SpriteId(505);
    pub const SPR_LIMPCITY_EYE: SpriteId = SpriteId(506);
    pub const SPR_LIMPCITY_EYE_CHAIN: SpriteId = SpriteId(507);
    pub const SPR_LIMPCITY_EYE_HINT: SpriteId = SpriteId(508);
    pub const SPR_LIMPCITY_EYE_RECHARGE: SpriteId = SpriteId(509);
    pub const SPR_LIMPCITY_EYELID: SpriteId = SpriteId(510);
    pub const SPR_LIMPCITY_HIDEEYE: SpriteId = SpriteId(511);
    pub const SPR_LOBBY_BARS: SpriteId = SpriteId(513);
    pub const SPR_LOBBY_CHATBOX: SpriteId = SpriteId(514);
    pub const SPR_LOBBY_DICON: SpriteId = SpriteId(515);
    pub const SPR_LOBBY_EXEICON: SpriteId = SpriteId(516);
    pub const SPR_LOBBY_EXEICON2: SpriteId = SpriteId(517);
    pub const SPR_LOBBY_EXEICON3: SpriteId = SpriteId(518);
    pub const SPR_LOBBY_EXEICON4: SpriteId = SpriteId(519);
    pub const SPR_LOBBY_EXEICON5: SpriteId = SpriteId(520);
    pub const SPR_LOBBY_ICON: SpriteId = SpriteId(521);
    pub const SPR_LOBBY_ICON_ARROW: SpriteId = SpriteId(522);
    pub const SPR_LOBBY_ICON_USED: SpriteId = SpriteId(523);
    pub const SPR_LOGO_LOGOS: SpriteId = SpriteId(525);
    pub const SPR_MAJIN_FOREST_SLOPE: SpriteId = SpriteId(527);
    pub const SPR_MAPVOTE: SpriteId = SpriteId(531);
    pub const SPR_MARIJUNA_BOOHOO: SpriteId = SpriteId(532);
    pub const SPR_MARIJUNA_CRYSTAL: SpriteId = SpriteId(533);
    pub const SPR_MARIJUNA_CRYSTAL2: SpriteId = SpriteId(534);
    pub const SPR_MARIJUNA_STATUE: SpriteId = SpriteId(535);
    pub const SPR_MARIJUNA_STATUE2: SpriteId = SpriteId(536);
    pub const SPR_MARIJUNA_FIREBALL: SpriteId = SpriteId(538);
    pub const SPR_MARIJUNA_JUDGER: SpriteId = SpriteId(539);
    pub const SPR_MARIJUNA_LAVAPLATFORM: SpriteId = SpriteId(542);
    pub const SPR_MARIJUNA_SLOPES: SpriteId = SpriteId(543);
    pub const SPR_MENU_AMY: SpriteId = SpriteId(547);
    pub const SPR_MENU_BARS: SpriteId = SpriteId(548);
    pub const SPR_MENU_CHAOS: SpriteId = SpriteId(549);
    pub const SPR_MENU_CHARACTERS: SpriteId = SpriteId(550);
    pub const SPR_MENU_CREAM: SpriteId = SpriteId(552);
    pub const SPR_MENU_EGG: SpriteId = SpriteId(555);
    pub const SPR_MENU_EXE: SpriteId = SpriteId(557);
    pub const SPR_MENU_EXELLER: SpriteId = SpriteId(558);
    pub const SPR_MENU_EXETIOR: SpriteId = SpriteId(559);
    pub const SPR_MENU_ICON: SpriteId = SpriteId(561);
    pub const SPR_MENU_KNUX: SpriteId = SpriteId(562);
    pub const SPR_MENU_LINKS: SpriteId = SpriteId(563);
    pub const SPR_MENU_LINKS2: SpriteId = SpriteId(564);
    pub const SPR_MENU_ICONFRAME: SpriteId = SpriteId(565);
    pub const SPR_MENU_PARTICLE: SpriteId = SpriteId(567);
    pub const SPR_MENU_PUBLIC: SpriteId = SpriteId(568);
    pub const SPR_MENU_PUBLIC2: SpriteId = SpriteId(569);
    pub const SPR_MENU_SALLY: SpriteId = SpriteId(573);
    pub const SPR_MENU_SKIN: SpriteId = SpriteId(574);
    pub const SPR_MENU_SLIDER: SpriteId = SpriteId(575);
    pub const SPR_MENU_SPIN: SpriteId = SpriteId(576);
    pub const SPR_MENU_TAILS: SpriteId = SpriteId(579);
    pub const SPR_MENU_TEXTBOX: SpriteId = SpriteId(580);
    pub const SPR_MENU_TEXTPANEL: SpriteId = SpriteId(581);
    pub const SPR_MENU_WAITING: SpriteId = SpriteId(582);
    pub const SPR_MERFURMU: SpriteId = SpriteId(584);
    pub const SPR_MERFURMU2: SpriteId = SpriteId(585);
    pub const SPR_MINI_BORDER: SpriteId = SpriteId(586);
    pub const SPR_MINISNOC: SpriteId = SpriteId(587);
    pub const SPR_MINISNOC_CHARG: SpriteId = SpriteId(588);
    pub const SPR_MINISNOC_GRAVEYARD: SpriteId = SpriteId(589);
    pub const SPR_MINISNOC_HIT: SpriteId = SpriteId(590);
    pub const SPR_MINISNOC_MCBIGTASTY: SpriteId = SpriteId(591);
    pub const SPR_MINISNOC_SPIN: SpriteId = SpriteId(592);
    pub const SPR_MOBILE_BUTTON: SpriteId = SpriteId(593);
    pub const SPR_MOBILE_DPAD: SpriteId = SpriteId(594);
    pub const SPR_MOBILE_SMALLBUTTON: SpriteId = SpriteId(595);
    pub const SPR_MOBILE_SMALLICONS: SpriteId = SpriteId(992);
    pub const SPR_NAP_ICEBLOCK: SpriteId = SpriteId(597);
    pub const SPR_NAP_ICEBLOCK2: SpriteId = SpriteId(598);
    pub const SPR_NAP_ICEBLOCK3: SpriteId = SpriteId(599);
    pub const SPR_NAP_ICEBLOCK_PART: SpriteId = SpriteId(600);
    pub const SPR_NAP_ICESPIKE: SpriteId = SpriteId(601);
    pub const SPR_NAP_SNOW: SpriteId = SpriteId(602);
    pub const SPR_NAP_SNOWBALL: SpriteId = SpriteId(603);
    pub const SPR_NAP_SNOWBALL_PART: SpriteId = SpriteId(604);
    pub const SPR_NONE: SpriteId = SpriteId(608);
    pub const SPR_MENU_WARNING: SpriteId = SpriteId(613);
    pub const SPR_MENU_WARNING2: SpriteId = SpriteId(614);
    pub const SPR_PET_ANNETTE: SpriteId = SpriteId(615);
    pub const SPR_PET_BALS: SpriteId = SpriteId(616);
    pub const SPR_PET_CATWE: SpriteId = SpriteId(617);
    pub const SPR_PET_CHAO: SpriteId = SpriteId(618);
    pub const SPR_PET_DALDOL: SpriteId = SpriteId(619);
    pub const SPR_PET_DALDOL_B: SpriteId = SpriteId(620);
    pub const SPR_PET_DANA: SpriteId = SpriteId(621);
    pub const SPR_PET_EGG: SpriteId = SpriteId(622);
    pub const SPR_PET_FLICKY: SpriteId = SpriteId(623);
    pub const SPR_PET_HAMTER: SpriteId = SpriteId(624);
    pub const SPR_PET_MAJIN: SpriteId = SpriteId(625);
    pub const SPR_PET_MERJONG: SpriteId = SpriteId(626);
    pub const SPR_PET_METAL: SpriteId = SpriteId(627);
    pub const SPR_PET_MKNUX: SpriteId = SpriteId(628);
    pub const SPR_PET_MOONWATER: SpriteId = SpriteId(629);
    pub const SPR_PET_MRPIXEL: SpriteId = SpriteId(630);
    pub const SPR_PET_NONE: SpriteId = SpriteId(631);
    pub const SPR_PET_PATOS: SpriteId = SpriteId(632);
    pub const SPR_PET_PERDIS: SpriteId = SpriteId(633);
    pub const SPR_PET_PHILNUX: SpriteId = SpriteId(634);
    pub const SPR_PET_SELFINJ: SpriteId = SpriteId(635);
    pub const SPR_PET_SEWERS: SpriteId = SpriteId(636);
    pub const SPR_PET_SKULL: SpriteId = SpriteId(637);
    pub const SPR_PET_SNIC: SpriteId = SpriteId(638);
    pub const SPR_PET_STOR: SpriteId = SpriteId(639);
    pub const SPR_PET_TITS: SpriteId = SpriteId(640);
    pub const SPR_PET_TRIZ1: SpriteId = SpriteId(641);
    pub const SPR_PET_TRIZ2: SpriteId = SpriteId(642);
    pub const SPR_PET_TRIZ3: SpriteId = SpriteId(643);
    pub const SPR_PET_UNCLE: SpriteId = SpriteId(644);
    pub const SPR_PET_WHISPER: SpriteId = SpriteId(645);
    pub const SPR_PF_LIFT: SpriteId = SpriteId(646);
    pub const SPR_PLATFORM: SpriteId = SpriteId(648);
    pub const SPR_PLAYERESCAPED: SpriteId = SpriteId(649);
    pub const SPR_PLAYERHEALTH: SpriteId = SpriteId(650);
    pub const SPR_PLAYERHEALTH_DEMON: SpriteId = SpriteId(651);
    pub const SPR_PLAYERHEALTH_HIT: SpriteId = SpriteId(652);
    pub const SPR_PLAYERHEALTH_REDRING: SpriteId = SpriteId(653);
    pub const SPR_PREVIEW_ACT9_1: SpriteId = SpriteId(654);
    pub const SPR_PREVIEW_ACT9_2: SpriteId = SpriteId(655);
    pub const SPR_PREVIEW_ACT9_3: SpriteId = SpriteId(656);
    pub const SPR_PREVIEW_DARK_TOWER_1: SpriteId = SpriteId(657);
    pub const SPR_PREVIEW_DARK_TOWER_2: SpriteId = SpriteId(658);
    pub const SPR_PREVIEW_DARK_TOWER_3: SpriteId = SpriteId(659);
    pub const SPR_PREVIEW_DOTDOTDOT_1: SpriteId = SpriteId(660);
    pub const SPR_PREVIEW_DOTDOTDOT_2: SpriteId = SpriteId(661);
    pub const SPR_PREVIEW_DOTDOTDOT_3: SpriteId = SpriteId(662);
    pub const SPR_PREVIEW_DESERT_TOWN_1: SpriteId = SpriteId(663);
    pub const SPR_PREVIEW_DESERT_TOWN_2: SpriteId = SpriteId(664);
    pub const SPR_PREVIEW_DESERT_TOWN_3: SpriteId = SpriteId(665);
    pub const SPR_PREVIEW_DESERT_TOWN_BACKGROUND: SpriteId = SpriteId(666);
    pub const SPR_PREVIEW_GREEN_HILL_1: SpriteId = SpriteId(667);
    pub const SPR_PREVIEW_GREEN_HILL_2: SpriteId = SpriteId(668);
    pub const SPR_PREVIEW_GREEN_HILL_3: SpriteId = SpriteId(669);
    pub const SPR_PREVIEW_GREEN_HILL_BACKGROUND: SpriteId = SpriteId(670);
    pub const SPR_PREVIEW_HAUNTING_DREAM_1: SpriteId = SpriteId(671);
    pub const SPR_PREVIEW_HAUNTING_DREAM_2: SpriteId = SpriteId(672);
    pub const SPR_PREVIEW_HAUNTING_DREAM_3: SpriteId = SpriteId(673);
    pub const SPR_PREVIEW_HAUNTING_DREAM_BACKGROUND: SpriteId = SpriteId(674);
    pub const SPR_PREVIEW_ANGEL_ISLAND_1: SpriteId = SpriteId(675);
    pub const SPR_PREVIEW_ANGEL_ISLAND_2: SpriteId = SpriteId(676);
    pub const SPR_PREVIEW_ANGEL_ISLAND_3: SpriteId = SpriteId(677);
    pub const SPR_PREVIEW_ANGEL_ISLAND_BACKGROUND: SpriteId = SpriteId(678);
    pub const SPR_PREVIEW_HIDE_AND_SEEK_2_1: SpriteId = SpriteId(679);
    pub const SPR_PREVIEW_HIDE_AND_SEEK_2_2: SpriteId = SpriteId(680);
    pub const SPR_PREVIEW_HIDE_AND_SEEK_2_3: SpriteId = SpriteId(681);
    pub const SPR_PREVIEW_HIDE_AND_SEEK_2_BACKGROUND: SpriteId = SpriteId(682);
    pub const SPR_PREVIEW_KIND_AND_FAIR_1: SpriteId = SpriteId(683);
    pub const SPR_PREVIEW_KIND_AND_FAIR_2: SpriteId = SpriteId(684);
    pub const SPR_PREVIEW_KIND_AND_FAIR_3: SpriteId = SpriteId(685);
    pub const SPR_PREVIEW_KIND_AND_FAIR_BACKGROUND: SpriteId = SpriteId(686);
    pub const SPR_PREVIEW_LEVEL1_1: SpriteId = SpriteId(989);
    pub const SPR_PREVIEW_LEVEL1_2: SpriteId = SpriteId(990);
    pub const SPR_PREVIEW_LEVEL1_3: SpriteId = SpriteId(991);
    pub const SPR_PREVIEW_LIMP_CITY_1: SpriteId = SpriteId(687);
    pub const SPR_PREVIEW_LIMP_CITY_2: SpriteId = SpriteId(688);
    pub const SPR_PREVIEW_LIMP_CITY_3: SpriteId = SpriteId(689);
    pub const SPR_PREVIEW_LIMP_CITY_BACKGROUND: SpriteId = SpriteId(690);
    pub const SPR_PREVIEW_MARIJUNA_1: SpriteId = SpriteId(691);
    pub const SPR_PREVIEW_MARIJUNA_2: SpriteId = SpriteId(692);
    pub const SPR_PREVIEW_MARIJUNA_3: SpriteId = SpriteId(693);
    pub const SPR_PREVIEW_MARIJUNA_BACKGROUND: SpriteId = SpriteId(694);
    pub const SPR_PREVIEW_MAJIN_FOREST_1: SpriteId = SpriteId(695);
    pub const SPR_PREVIEW_MAJIN_FOREST_2: SpriteId = SpriteId(696);
    pub const SPR_PREVIEW_MAJIN_FOREST_3: SpriteId = SpriteId(697);
    pub const SPR_PREVIEW_MAJIN_FOREST_BACKGROUND: SpriteId = SpriteId(698);
    pub const SPR_PREVIEW_NASTY_PARADISE_1: SpriteId = SpriteId(699);
    pub const SPR_PREVIEW_NASTY_PARADISE_2: SpriteId = SpriteId(700);
    pub const SPR_PREVIEW_NASTY_PARADISE_3: SpriteId = SpriteId(701);
    pub const SPR_PREVIEW_NASTY_PARADISE_BACKGROUND: SpriteId = SpriteId(702);
    pub const SPR_PREVIEW_NOT_PERFECT_1: SpriteId = SpriteId(703);
    pub const SPR_PREVIEW_NOT_PERFECT_2: SpriteId = SpriteId(704);
    pub const SPR_PREVIEW_NOT_PERFECT_3: SpriteId = SpriteId(705);
    pub const SPR_PREVIEW_NOT_PERFECT_4: SpriteId = SpriteId(706);
    pub const SPR_PREVIEW_NOT_PERFECT_BACKGROUND: SpriteId = SpriteId(707);
    pub const SPR_PREVIEW_PRICELESS_FREEDOM_1: SpriteId = SpriteId(708);
    pub const SPR_PREVIEW_PRICELESS_FREEDOM_2: SpriteId = SpriteId(709);
    pub const SPR_PREVIEW_PRICELESS_FREEDOM_3: SpriteId = SpriteId(710);
    pub const SPR_PREVIEW_RAVINE_MIST_1: SpriteId = SpriteId(711);
    pub const SPR_PREVIEW_RAVINE_MIST_2: SpriteId = SpriteId(712);
    pub const SPR_PREVIEW_RAVINE_MIST_BACKGROUND: SpriteId = SpriteId(713);
    pub const SPR_PREVIEW_TORTURE_CAVE_1: SpriteId = SpriteId(714);
    pub const SPR_PREVIEW_TORTURE_CAVE_2: SpriteId = SpriteId(715);
    pub const SPR_PREVIEW_TORTURE_CAVE_3: SpriteId = SpriteId(716);
    pub const SPR_PREVIEW_TORTURE_CAVE_BACKGROUND: SpriteId = SpriteId(717);
    pub const SPR_PREVIEW_VOLCANO_VALLEY_1: SpriteId = SpriteId(718);
    pub const SPR_PREVIEW_VOLCANO_VALLEY_2: SpriteId = SpriteId(719);
    pub const SPR_PREVIEW_VOLCANO_VALLEY_3: SpriteId = SpriteId(720);
    pub const SPR_PREVIEW_VOLCANO_VALLEY_BACKGROUND: SpriteId = SpriteId(721);
    pub const SPR_PREVIEW_WEED_ZONE_1: SpriteId = SpriteId(722);
    pub const SPR_PREVIEW_WEED_ZONE_2: SpriteId = SpriteId(723);
    pub const SPR_PREVIEW_WEED_ZONE_3: SpriteId = SpriteId(724);
    pub const SPR_PREVIEW_WEED_ZONE_BACKGROUND: SpriteId = SpriteId(725);
    pub const SPR_PREVIEW_YOU_CANT_RUN_1: SpriteId = SpriteId(726);
    pub const SPR_PREVIEW_YOU_CANT_RUN_2: SpriteId = SpriteId(727);
    pub const SPR_PREVIEW_YOU_CANT_RUN_3: SpriteId = SpriteId(728);
    pub const SPR_PREVIEW_YOU_CANT_RUN_BACKGROUND: SpriteId = SpriteId(729);
    pub const SPR_PREVIEW_FART_ZONE_1: SpriteId = SpriteId(730);
    pub const SPR_PREVIEW_FART_ZONE_2: SpriteId = SpriteId(731);
    pub const SPR_PREVIEW_FART_ZONE_3: SpriteId = SpriteId(732);
    pub const SPR_RAMPHELPER_RIGHT: SpriteId = SpriteId(733);
    pub const SPR_RAVINEMIST_BUSH: SpriteId = SpriteId(736);
    pub const SPR_RAVINEMIST_BUSH_ANIMATED: SpriteId = SpriteId(737);
    pub const SPR_RAVINEMIST_SHARD: SpriteId = SpriteId(740);
    pub const SPR_RAVINEMIST_SONIC: SpriteId = SpriteId(741);
    pub const SPR_RAVINEMIST_SONIC2: SpriteId = SpriteId(742);
    pub const SPR_RAVINEMIST_SONIC3: SpriteId = SpriteId(743);
    pub const SPR_RAVINEMIST_SONICDEAD: SpriteId = SpriteId(744);
    pub const SPR_RAVINEMIST_UI: SpriteId = SpriteId(745);
    pub const SPR_RED: SpriteId = SpriteId(746);
    pub const SPR_REDRING: SpriteId = SpriteId(747);
    pub const SPR_REDRING_FORE: SpriteId = SpriteId(748);
    pub const SPR_RESULTS_BAR: SpriteId = SpriteId(749);
    pub const SPR_RESULTS_ICONS: SpriteId = SpriteId(750);
    pub const SPR_REVIVAL: SpriteId = SpriteId(751);
    pub const SPR_REVIVAL2: SpriteId = SpriteId(752);
    pub const SPR_RING: SpriteId = SpriteId(753);
    pub const SPR_RING_SPARKLE: SpriteId = SpriteId(754);
    pub const SPR_RING_TELEPORT: SpriteId = SpriteId(755);
    pub const SPR_RINGLOSE: SpriteId = SpriteId(756);
    pub const SPR_RINGPART: SpriteId = SpriteId(757);
    pub const SPR_RINGPART2: SpriteId = SpriteId(758);
    pub const SPR_RINGPART3: SpriteId = SpriteId(759);
    pub const SPR_RINGPART4: SpriteId = SpriteId(760);
    pub const SPR_RINGSPAWN: SpriteId = SpriteId(761);
    pub const SPR_ROSEHEART: SpriteId = SpriteId(762);
    pub const SPR_SALLY_ATTACK: SpriteId = SpriteId(763);
    pub const SPR_SALLY_BALANCING: SpriteId = SpriteId(764);
    pub const SPR_SALLY_DEAD: SpriteId = SpriteId(765);
    pub const SPR_SALLY_EMOTION1: SpriteId = SpriteId(766);
    pub const SPR_SALLY_EMOTION2: SpriteId = SpriteId(767);
    pub const SPR_SALLY_EMOTION3: SpriteId = SpriteId(768);
    pub const SPR_SALLY_FALL: SpriteId = SpriteId(769);
    pub const SPR_SALLY_HURT: SpriteId = SpriteId(770);
    pub const SPR_SALLY_IDLE: SpriteId = SpriteId(771);
    pub const SPR_SALLY_JUMP: SpriteId = SpriteId(772);
    pub const SPR_SALLY_LOOKDOWN: SpriteId = SpriteId(773);
    pub const SPR_SALLY_LOOKUP: SpriteId = SpriteId(774);
    pub const SPR_SALLY_RUN: SpriteId = SpriteId(775);
    pub const SPR_SALLY_SLIDE: SpriteId = SpriteId(776);
    pub const SPR_SALLY_WALK: SpriteId = SpriteId(777);
    pub const SPR_SALLY_ZIPLINE: SpriteId = SpriteId(778);
    pub const SPR_SALLYSHIELD: SpriteId = SpriteId(779);
    pub const SPR_SALLYSHIELD2: SpriteId = SpriteId(780);
    pub const SPR_SCREENOVERLAY: SpriteId = SpriteId(781);
    pub const SPR_SCREENOVERLAY2: SpriteId = SpriteId(782);
    pub const SPR_SELFINSERT: SpriteId = SpriteId(783);
    pub const SPR_SELFINSERT2: SpriteId = SpriteId(784);
    pub const SPR_SENSOR: SpriteId = SpriteId(785);
    pub const SPR_SHARD_SPARKLE: SpriteId = SpriteId(786);
    pub const SPR_SHIELDBREAK: SpriteId = SpriteId(787);
    pub const SPR_SHIELDBREAK2: SpriteId = SpriteId(788);
    pub const SPR_SHOCKPARTICLE: SpriteId = SpriteId(789);
    pub const SPR_SINDICATOR: SpriteId = SpriteId(790);
    pub const SPR_SINDICATOR2: SpriteId = SpriteId(791);
    pub const SPR_SINDICATOR3: SpriteId = SpriteId(792);
    pub const SPR_SMOKE: SpriteId = SpriteId(793);
    pub const SPR_SNBWAY: SpriteId = SpriteId(794);
    pub const SPR_SOTA: SpriteId = SpriteId(795);
    pub const SPR_SOTA2: SpriteId = SpriteId(796);
    pub const SPR_SOTA3: SpriteId = SpriteId(797);
    pub const SPR_SOUNDEMITTER: SpriteId = SpriteId(798);
    pub const SPR_SPAWNPOINT: SpriteId = SpriteId(799);
    pub const SPR_SPIKE: SpriteId = SpriteId(800);
    pub const SPR_SPIKE_MOVING: SpriteId = SpriteId(801);
    pub const SPR_SPIKE_R: SpriteId = SpriteId(802);
    pub const SPR_SPRING_LEFT: SpriteId = SpriteId(803);
    pub const SPR_SPRING_RIGHT: SpriteId = SpriteId(804);
    pub const SPR_SPRING_UP: SpriteId = SpriteId(805);
    pub const SPR_STATIC: SpriteId = SpriteId(806);
    pub const SPR_STATUS: SpriteId = SpriteId(807);
    pub const SPR_SUDDEN_DEATH: SpriteId = SpriteId(808);
    pub const SPR_SURVIVORSESCAPED: SpriteId = SpriteId(809);
    pub const SPR_TAB_ENTRY: SpriteId = SpriteId(810);
    pub const SPR_TAIL_RAY_DEAD: SpriteId = SpriteId(811);
    pub const SPR_TAIL_RAY_DEAD1: SpriteId = SpriteId(812);
    pub const SPR_TAIL_RAY_DEAD2: SpriteId = SpriteId(813);
    pub const SPR_TAIL_RAY_DEAD3: SpriteId = SpriteId(814);
    pub const SPR_TAIL_RAY_DEAD4: SpriteId = SpriteId(815);
    pub const SPR_TAILS_ATTACK1: SpriteId = SpriteId(816);
    pub const SPR_TAILS_BALANCING: SpriteId = SpriteId(817);
    pub const SPR_TAILS_DEAD: SpriteId = SpriteId(818);
    pub const SPR_TAILS_EMOTION1: SpriteId = SpriteId(819);
    pub const SPR_TAILS_EMOTION2: SpriteId = SpriteId(820);
    pub const SPR_TAILS_EMOTION3: SpriteId = SpriteId(821);
    pub const SPR_TAILS_FALL: SpriteId = SpriteId(822);
    pub const SPR_TAILS_FLY: SpriteId = SpriteId(823);
    pub const SPR_TAILS_HURT: SpriteId = SpriteId(824);
    pub const SPR_TAILS_IDLE: SpriteId = SpriteId(825);
    pub const SPR_TAILS_JUMP: SpriteId = SpriteId(826);
    pub const SPR_TAILS_LOOKDOWN: SpriteId = SpriteId(827);
    pub const SPR_TAILS_LOOKUP: SpriteId = SpriteId(828);
    pub const SPR_TAILS_RAY: SpriteId = SpriteId(829);
    pub const SPR_TAILS_RUN: SpriteId = SpriteId(830);
    pub const SPR_TAILS_TAIL1: SpriteId = SpriteId(831);
    pub const SPR_TAILS_TAIL2: SpriteId = SpriteId(832);
    pub const SPR_TAILS_WALK: SpriteId = SpriteId(833);
    pub const SPR_TAILS_ZIPLINE: SpriteId = SpriteId(834);
    pub const SPR_TAILSCHARGE: SpriteId = SpriteId(835);
    pub const SPR_TIMEOVER: SpriteId = SpriteId(836);
    pub const SPR_TITLECARD1: SpriteId = SpriteId(837);
    pub const SPR_TITLECARD2: SpriteId = SpriteId(838);
    pub const SPR_VOTE_BARS: SpriteId = SpriteId(839);
    pub const SPR_VV_BARREL: SpriteId = SpriteId(840);
    pub const SPR_VV_BARREL2: SpriteId = SpriteId(841);
    pub const SPR_VV_LAVACOLUMN: SpriteId = SpriteId(844);
    pub const SPR_VV_LAVACOLUMN2: SpriteId = SpriteId(845);
    pub const SPR_VV_VASE: SpriteId = SpriteId(849);
    pub const SPR_VV_VASEPIECE: SpriteId = SpriteId(850);
    pub const SPR_WARNING: SpriteId = SpriteId(851);
    pub const SPR_WATERSPLASH: SpriteId = SpriteId(852);
    pub const SPR_WEED_ARMS: SpriteId = SpriteId(853);
    pub const SPR_WEED_CHAIN: SpriteId = SpriteId(854);
    pub const SPR_WEED_CHAIN2: SpriteId = SpriteId(855);
    pub const SPR_WEED_CONVEYOR: SpriteId = SpriteId(856);
    pub const SPR_WEED_CONVEYOR2: SpriteId = SpriteId(857);
    pub const SPR_WEED_CONVEYOR3: SpriteId = SpriteId(858);
    pub const SPR_WEED_LATERN: SpriteId = SpriteId(859);
    pub const SPR_WEED_LIGHT: SpriteId = SpriteId(860);
    pub const SPR_WEED_GHOSTS: SpriteId = SpriteId(861);
    pub const SPR_WEED_GHOSTS2: SpriteId = SpriteId(862);
    pub const SPR_WEED_SLOPE_JUMPTHROUGH: SpriteId = SpriteId(863);
    pub const SPR_WEED_SLOPE: SpriteId = SpriteId(864);
    pub const SPR_WEED_VIGNETTE: SpriteId = SpriteId(869);
    pub const SPR_WHITE: SpriteId = SpriteId(870);
    pub const SPR_YCR_SLOPE: SpriteId = SpriteId(871);
    pub const SPR_YCR_SLOPE2: SpriteId = SpriteId(872);
    pub const SPR_YOUKILLEDEVERYONE: SpriteId = SpriteId(875);
    pub const SPR_YSPRING_UP: SpriteId = SpriteId(876);

    /// The direction arrows the ability signs had drawn in after their keys (this
    /// port's own, cut out of spr_gui_chaosattack and spr_gui_chaoswalldash).
    pub const SPR_GUI_HINT_DIRECTIONS: SpriteId = SpriteId(895);
    pub const SPR_RAVINEMIST_SHARD_WITHOUT_GLOW: SpriteId = SpriteId(896);
    /// Our own smooth colored noise that the underwater filter of Torture Cave shimmers with.
    pub const SPR_UNDERWATER_NOISE: SpriteId = SpriteId(982);
    /// Our own: one dark cell of Fart Zone's grid, without the floor tile's edge line,
    /// behind its map preview.
    pub const SPR_PREVIEW_FART_ZONE_BACKGROUND: SpriteId = SpriteId(983);
}

pub mod sound {
    use super::SoundId;
    pub const MUS_ACT9: SoundId = SoundId(0);
    pub const MUS_ACT9_CHASE: SoundId = SoundId(1);
    pub const MUS_ANGELISLAND: SoundId = SoundId(2);
    pub const MUS_ANGELISLAND_CHASE: SoundId = SoundId(3);
    pub const MUS_CHASE: SoundId = SoundId(212);
    pub const MUS_DARKTOWER: SoundId = SoundId(5);
    pub const MUS_DARKTOWER_CHASE: SoundId = SoundId(6);
    pub const MUS_DESERTTOWN: SoundId = SoundId(7);
    pub const MUS_DESERTTOWN_CHASE: SoundId = SoundId(8);
    pub const MUS_DOTDOTDOT: SoundId = SoundId(9);
    pub const MUS_DOTDOTDOT2: SoundId = SoundId(10);
    pub const MUS_DOTDOTDOT_CHASE: SoundId = SoundId(11);
    pub const MUS_EXEWIN: SoundId = SoundId(12);
    pub const MUS_FARTZONE: SoundId = SoundId(13);
    pub const MUS_FARTZONE_CHASE: SoundId = SoundId(14);
    pub const MUS_HAUNTDREAM: SoundId = SoundId(15);
    pub const MUS_HAUNTDREAM_CHASE: SoundId = SoundId(16);
    pub const MUS_HIDEANDSEEK2: SoundId = SoundId(17);
    pub const MUS_HIDEANDSEEK2_CHASE: SoundId = SoundId(18);
    pub const MUS_HILL: SoundId = SoundId(19);
    pub const MUS_HILL_CHASE: SoundId = SoundId(20);
    pub const MUS_KINDANDFAIR: SoundId = SoundId(21);
    pub const MUS_KINDANDFAIR_CHASE: SoundId = SoundId(22);
    pub const MUS_LEVEL: SoundId = SoundId(213);
    pub const MUS_LIMPCITY: SoundId = SoundId(23);
    pub const MUS_LIMPCITY_CHASE: SoundId = SoundId(24);
    pub const MUS_LOBBY: SoundId = SoundId(25);
    pub const MUS_LOBBY_EPIC: SoundId = SoundId(26);
    pub const MUS_LOGO: SoundId = SoundId(27);
    pub const MUS_MAJINFOREST: SoundId = SoundId(28);
    pub const MUS_MAJINFOREST_CHASE: SoundId = SoundId(29);
    pub const MUS_MARIJUNA: SoundId = SoundId(30);
    pub const MUS_MARIJUNA_CHASE: SoundId = SoundId(31);
    pub const MUS_MENU: SoundId = SoundId(32);
    pub const MUS_MINDFUCK: SoundId = SoundId(33);
    pub const MUS_MINIGAME: SoundId = SoundId(34);
    pub const MUS_NASTYPARADISE: SoundId = SoundId(35);
    pub const MUS_NASTYPARADISE_CHASE: SoundId = SoundId(36);
    pub const MUS_NOTPERFECT: SoundId = SoundId(37);
    pub const MUS_NOTPERFECT_CHASE: SoundId = SoundId(38);
    pub const MUS_PRICELESSFREEDOM: SoundId = SoundId(39);
    pub const MUS_PRICELESSFREEDOM_CHASE: SoundId = SoundId(40);
    pub const MUS_RAVIMEMIST: SoundId = SoundId(41);
    pub const MUS_RAVIMEMIST_CHASE: SoundId = SoundId(42);
    pub const MUS_SURVWIN: SoundId = SoundId(43);
    pub const MUS_TIMEOVER: SoundId = SoundId(44);
    pub const MUS_TORTURECAVE: SoundId = SoundId(45);
    pub const MUS_TORTURECAVE_CHASE: SoundId = SoundId(46);
    pub const MUS_VOLCANOVALLEY: SoundId = SoundId(47);
    pub const MUS_VOLCANOVALLEY_CHASE: SoundId = SoundId(48);
    pub const MUS_WAITING: SoundId = SoundId(49);
    pub const MUS_WEEDZONE: SoundId = SoundId(50);
    pub const MUS_WEEDZONE_CHASE: SoundId = SoundId(51);
    pub const MUS_YOUCANTRUN: SoundId = SoundId(52);
    pub const MUS_YOUCANTRUN_CHASE: SoundId = SoundId(53);
    pub const SND_ACHIVEMENT: SoundId = SoundId(54);
    pub const SND_ACID: SoundId = SoundId(55);
    pub const SND_BLACKRING: SoundId = SoundId(56);
    pub const SND_BLACKRING_BALL: SoundId = SoundId(57);
    pub const SND_BOOHOO: SoundId = SoundId(58);
    pub const SND_BOOM: SoundId = SoundId(59);
    pub const SND_BOS: SoundId = SoundId(60);
    pub const SND_BREAK: SoundId = SoundId(61);
    pub const SND_BUBLE: SoundId = SoundId(62);
    pub const SND_CHAOS_ATTACK: SoundId = SoundId(63);
    pub const SND_CHAOS_DASH: SoundId = SoundId(64);
    pub const SND_CHAOS_KILL: SoundId = SoundId(65);
    pub const SND_CHAOS_KILL2: SoundId = SoundId(66);
    pub const SND_CHAOS_KILL3: SoundId = SoundId(67);
    pub const SND_CHAOS_KILL4: SoundId = SoundId(68);
    pub const SND_CHAOS_KILL5: SoundId = SoundId(69);
    pub const SND_CHAOS_KILL6: SoundId = SoundId(70);
    pub const SND_CHAOS_KILL7: SoundId = SoundId(71);
    pub const SND_CHAOS_KILL8: SoundId = SoundId(72);
    pub const SND_CHAOS_LAND: SoundId = SoundId(73);
    pub const SND_CHAOS_LAUGH: SoundId = SoundId(74);
    pub const SND_CHAOS_PIZZA: SoundId = SoundId(75);
    pub const SND_CHAOS_STUN: SoundId = SoundId(76);
    pub const SND_CHAOS_STURN: SoundId = SoundId(77);
    pub const SND_CHAOS_TAUNT: SoundId = SoundId(78);
    pub const SND_CHAOS_TAUNT2: SoundId = SoundId(79);
    pub const SND_CHAOS_VINEBOOM: SoundId = SoundId(80);
    pub const SND_CLOCK: SoundId = SoundId(81);
    pub const SND_CREAMDASH: SoundId = SoundId(82);
    pub const SND_CREAMRING: SoundId = SoundId(83);
    pub const SND_DASH: SoundId = SoundId(84);
    pub const SND_DEAD: SoundId = SoundId(85);
    pub const SND_DEMONIZATION: SoundId = SoundId(86);
    pub const SND_DESTINY: SoundId = SoundId(87);
    pub const SND_DOOR: SoundId = SoundId(88);
    pub const SND_DUMMY: SoundId = SoundId(89);
    pub const SND_DUMMY2: SoundId = SoundId(90);
    pub const SND_DUMMY3: SoundId = SoundId(91);
    pub const SND_ECHAIN: SoundId = SoundId(92);
    pub const SND_ECHAIN_PREPARE: SoundId = SoundId(93);
    pub const SND_EGG_DJUMP: SoundId = SoundId(94);
    pub const SND_EGG_SHIELD: SoundId = SoundId(95);
    pub const SND_EGG_TRACKER: SoundId = SoundId(96);
    pub const SND_EGG_TRACKER_ACTIVATE: SoundId = SoundId(97);
    pub const SND_ELECTROSHOCK: SoundId = SoundId(98);
    pub const SND_EXE_APPEAR: SoundId = SoundId(99);
    pub const SND_EXE_APPEAR2: SoundId = SoundId(100);
    pub const SND_EXE_APPEAR3: SoundId = SoundId(101);
    pub const SND_EXE_INVISENTER: SoundId = SoundId(102);
    pub const SND_EXE_INVISENTER2: SoundId = SoundId(103);
    pub const SND_EXE_KILL1: SoundId = SoundId(104);
    pub const SND_EXE_KILL2: SoundId = SoundId(105);
    pub const SND_EXE_KILL3: SoundId = SoundId(106);
    pub const SND_EXE_KILL4: SoundId = SoundId(107);
    pub const SND_EXE_LAUGH: SoundId = SoundId(108);
    pub const SND_EXE_RINGSHUTTER: SoundId = SoundId(109);
    pub const SND_EXE_STUN: SoundId = SoundId(110);
    pub const SND_EXE_STUN2: SoundId = SoundId(111);
    pub const SND_EXE_TAUNT: SoundId = SoundId(112);
    pub const SND_EXE_TAUNT1: SoundId = SoundId(113);
    pub const SND_EXE_TAUNT2: SoundId = SoundId(114);
    pub const SND_EXE_WINS: SoundId = SoundId(115);
    pub const SND_EXELLER_CLONE: SoundId = SoundId(116);
    pub const SND_EXELLER_CLONELINE: SoundId = SoundId(117);
    pub const SND_EXELLER_KILL1: SoundId = SoundId(118);
    pub const SND_EXELLER_KILL2: SoundId = SoundId(119);
    pub const SND_EXELLER_KILL3: SoundId = SoundId(120);
    pub const SND_EXELLER_KILL4: SoundId = SoundId(121);
    pub const SND_EXELLER_KILL5: SoundId = SoundId(122);
    pub const SND_EXELLER_KILL6: SoundId = SoundId(123);
    pub const SND_EXELLER_KILL7: SoundId = SoundId(124);
    pub const SND_EXELLER_LAUGH: SoundId = SoundId(125);
    pub const SND_EXELLER_STUN: SoundId = SoundId(126);
    pub const SND_EXELLER_TAUNT1: SoundId = SoundId(127);
    pub const SND_EXELLER_TAUNT2: SoundId = SoundId(128);
    pub const SND_EXELLER_TAUNT3: SoundId = SoundId(129);
    pub const SND_EXETIOR_KILL1: SoundId = SoundId(130);
    pub const SND_EXETIOR_KILL2: SoundId = SoundId(131);
    pub const SND_EXETIOR_KILL3: SoundId = SoundId(132);
    pub const SND_EXETIOR_KILL4: SoundId = SoundId(133);
    pub const SND_EXETIOR_KILL5: SoundId = SoundId(134);
    pub const SND_EXETIOR_KILL6: SoundId = SoundId(135);
    pub const SND_EXETIOR_LAUGH: SoundId = SoundId(136);
    pub const SND_EXETIOR_RING1: SoundId = SoundId(137);
    pub const SND_EXETIOR_RING2: SoundId = SoundId(138);
    pub const SND_EXETIOR_RING3: SoundId = SoundId(139);
    pub const SND_EXETIOR_RING4: SoundId = SoundId(140);
    pub const SND_EXETIOR_STOMP: SoundId = SoundId(142);
    pub const SND_EXETIOR_STOMPLAND: SoundId = SoundId(143);
    pub const SND_EXETIOR_STUN: SoundId = SoundId(144);
    pub const SND_EXETIOR_TAUNT1: SoundId = SoundId(145);
    pub const SND_EXETIOR_TAUNT2: SoundId = SoundId(146);
    pub const SND_EXETIOR_TAUNT3: SoundId = SoundId(147);
    pub const SND_FNAC: SoundId = SoundId(150);
    pub const SND_HEAL: SoundId = SoundId(151);
    pub const SND_HURT: SoundId = SoundId(152);
    pub const SND_ICE_BREAK: SoundId = SoundId(153);
    pub const SND_ICE_SPAWN: SoundId = SoundId(154);
    pub const SND_JUDGER: SoundId = SoundId(155);
    pub const SND_JUMP: SoundId = SoundId(156);
    pub const SND_LAVA: SoundId = SoundId(157);
    pub const SND_LAVAAPPEAR: SoundId = SoundId(158);
    pub const SND_LAVAHIT: SoundId = SoundId(159);
    pub const SND_LIFT: SoundId = SoundId(160);
    pub const SND_MENU_PRESS: SoundId = SoundId(161);
    pub const SND_MENU_SELECT: SoundId = SoundId(162);
    pub const SND_MESSAGE: SoundId = SoundId(164);
    pub const SND_MINIBAL: SoundId = SoundId(165);
    pub const SND_MINIDESTROY: SoundId = SoundId(166);
    pub const SND_MINIDIE: SoundId = SoundId(167);
    pub const SND_MINIEXTRA: SoundId = SoundId(168);
    pub const SND_MINISPIN: SoundId = SoundId(169);
    pub const SND_MOVINGSPIKE: SoundId = SoundId(171);
    pub const SND_NONO: SoundId = SoundId(172);
    pub const SND_NPTELEPORT: SoundId = SoundId(173);
    pub const SND_RAIN: SoundId = SoundId(174);
    pub const SND_READY: SoundId = SoundId(175);
    pub const SND_REDRING: SoundId = SoundId(176);
    pub const SND_RING: SoundId = SoundId(177);
    pub const SND_RINGABSORB: SoundId = SoundId(178);
    pub const SND_RINGLOSE: SoundId = SoundId(179);
    pub const SND_ROAR: SoundId = SoundId(180);
    pub const SND_SALLY_SHIELD: SoundId = SoundId(181);
    pub const SND_SALLY_SHIELDBREAK: SoundId = SoundId(182);
    pub const SND_SALLY_SLIDE: SoundId = SoundId(183);
    pub const SND_SHARD: SoundId = SoundId(184);
    pub const SND_STALACTITE_NOTICE: SoundId = SoundId(185);
    pub const SND_SLIME: SoundId = SoundId(186);
    pub const SND_SMOKE: SoundId = SoundId(187);
    pub const SND_SNAP: SoundId = SoundId(188);
    pub const SND_SNOWBALL_BREAK: SoundId = SoundId(189);
    pub const SND_SNOWBALL_ROLL: SoundId = SoundId(190);
    pub const SND_SPIKE: SoundId = SoundId(191);
    pub const SND_SPIN: SoundId = SoundId(192);
    pub const SND_SPRING: SoundId = SoundId(193);
    pub const SND_SPRING_REVERB: SoundId = SoundId(194);
    pub const SND_SUDDENDEATH: SoundId = SoundId(195);
    pub const SND_SURVIVOR_WIN: SoundId = SoundId(196);
    pub const SND_TAILS_CHARGE: SoundId = SoundId(197);
    pub const SND_TAILS_FLY: SoundId = SoundId(198);
    pub const SND_TAILS_SHOOT: SoundId = SoundId(200);
    pub const SND_TAILSBALL: SoundId = SoundId(201);
    pub const SND_TAILSBALL_CHASE: SoundId = SoundId(202);
    pub const SND_TAILSBALL_JUMPSCARE: SoundId = SoundId(203);
    pub const SND_TAILSBALL_JUMPSCARE2: SoundId = SoundId(204);
    pub const SND_TELEPORT: SoundId = SoundId(205);
    pub const SND_THUNDER: SoundId = SoundId(206);
    pub const SND_VASEBREAK: SoundId = SoundId(207);
    pub const SND_WATERDROPS: SoundId = SoundId(208);
    pub const SND_WATERSPLASH: SoundId = SoundId(209);
    pub const SND_WDARMS: SoundId = SoundId(210);
    pub const SND_WDLAMP: SoundId = SoundId(211);
}
