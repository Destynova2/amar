# Licences des données

## NOAA CO-OPS

Les constantes harmoniques NOAA CO-OPS, les datums de stations et les
prédictions officielles récupérées depuis `api.tidesandcurrents.noaa.gov` sont
des données du gouvernement des États-Unis et relèvent du domaine public.

Extraction M1 : 2026-07-06.

Stations M1 vérifiées via NOAA `mdapi` avant inclusion : chaque station est
`tidal=true`, expose `harmonicConstituents`, et son endpoint `harcon` retourne
37 constituants harmoniques en mètres.

| Station | `tideType` mdapi | Rôle de validation |
|---|---|---|
| `noaa:8443970` Boston | Mixed | Semi-diurne M0 |
| `noaa:9414290` San Francisco | Mixed | Mixte M0 |
| `noaa:8729840` Pensacola | Diurnal | Diurne M0 |
| `noaa:9447130` Seattle | Mixed | Témoin M0 hors réglage |
| `noaa:8410140` Eastport | Mixed | Grand marnage semi-diurne |
| `noaa:1612340` Honolulu | Mixed | Faible marnage |
| `noaa:8724580` Key West | Mixed | Faible marnage |
| `noaa:8771450` Galveston Pier 21 | Diurnal | Diurne supplémentaire |

## Fixtures M1

Extraction : 2026-07-06.

| Fichier | URL d'origine | SHA-256 |
|---|---|---|
| `fixtures/noaa/1612340/station.json` | `https://api.tidesandcurrents.noaa.gov/mdapi/prod/webapi/stations/1612340.json?expand=details` | `9e1027aac10ae9f1801d2a270d089d852232bc59d961e958c7a7fd36d2b387b7` |
| `fixtures/noaa/1612340/datums.json` | `https://api.tidesandcurrents.noaa.gov/mdapi/prod/webapi/stations/1612340/datums.json?units=metric` | `e304c1e1816d34e4f0683e19c25d3d75d937d18f3776186adff2029781d1112d` |
| `fixtures/noaa/1612340/harcon.json` | `https://api.tidesandcurrents.noaa.gov/mdapi/prod/webapi/stations/1612340/harcon.json?units=metric` | `26d9c8960e25f5fe4093b358c3b543d5c73ca83e2aa192698bc711f917cc58d7` |
| `fixtures/noaa/1612340/predictions_2026-08-15_2026-08-21.json` | `https://api.tidesandcurrents.noaa.gov/api/prod/datagetter?product=predictions&application=amar&begin_date=20260815&end_date=20260821&datum=MLLW&station=1612340&time_zone=gmt&units=metric&interval=h&format=json` | `1e8046a60180631d4c6fb2fd7e2cc64ce4afd71c8fa591f590f6f1040086dc74` |
| `fixtures/noaa/1612340/predictions_2031-08-15_2031-08-21.json` | `https://api.tidesandcurrents.noaa.gov/api/prod/datagetter?product=predictions&application=amar&begin_date=20310815&end_date=20310821&datum=MLLW&station=1612340&time_zone=gmt&units=metric&interval=h&format=json` | `a209bbd8cc6061682096d402b95d5f0c461e1d8f524fa2f38725d89f230adb52` |
| `fixtures/noaa/1612340/predictions_2036-08-15_2036-08-21.json` | `https://api.tidesandcurrents.noaa.gov/api/prod/datagetter?product=predictions&application=amar&begin_date=20360815&end_date=20360821&datum=MLLW&station=1612340&time_zone=gmt&units=metric&interval=h&format=json` | `4930ec38797a3768cbc733ae7ddc289fa22bfed079dedb04e791f0a5515e8d7e` |
| `fixtures/noaa/8410140/station.json` | `https://api.tidesandcurrents.noaa.gov/mdapi/prod/webapi/stations/8410140.json?expand=details` | `edb5f24e1a8ec390bfd4f6ac3575b09ca8c05362ed42a3f4a9db2edf2a8ec689` |
| `fixtures/noaa/8410140/datums.json` | `https://api.tidesandcurrents.noaa.gov/mdapi/prod/webapi/stations/8410140/datums.json?units=metric` | `63eff7cc22863ddbd745b4700208395636e2420bb0336dac44c8048951d19dc2` |
| `fixtures/noaa/8410140/harcon.json` | `https://api.tidesandcurrents.noaa.gov/mdapi/prod/webapi/stations/8410140/harcon.json?units=metric` | `2f1ed04e7768c15f0e7c62ec4dad0eab7d75de82050dc62ff61c814f70da18a4` |
| `fixtures/noaa/8410140/predictions_2026-08-15_2026-08-21.json` | `https://api.tidesandcurrents.noaa.gov/api/prod/datagetter?product=predictions&application=amar&begin_date=20260815&end_date=20260821&datum=MLLW&station=8410140&time_zone=gmt&units=metric&interval=h&format=json` | `4962b65233816a69f4a242d48ad3a8854cb7bb73df5d040a1021d4725cc7edb7` |
| `fixtures/noaa/8410140/predictions_2031-08-15_2031-08-21.json` | `https://api.tidesandcurrents.noaa.gov/api/prod/datagetter?product=predictions&application=amar&begin_date=20310815&end_date=20310821&datum=MLLW&station=8410140&time_zone=gmt&units=metric&interval=h&format=json` | `374de8110b229a3d636b2c3c44e16290799d5d1ec9b00baab09798514f1b4f54` |
| `fixtures/noaa/8410140/predictions_2036-08-15_2036-08-21.json` | `https://api.tidesandcurrents.noaa.gov/api/prod/datagetter?product=predictions&application=amar&begin_date=20360815&end_date=20360821&datum=MLLW&station=8410140&time_zone=gmt&units=metric&interval=h&format=json` | `91a279677dcab2541cabf7f41a74cb9aefda63da16d34479e73731054bcc48be` |
| `fixtures/noaa/8443970/station.json` | `https://api.tidesandcurrents.noaa.gov/mdapi/prod/webapi/stations/8443970.json?expand=details` | `52abdb362d8a1a6b11b6164a4e78df0a6593b99ee854f7edb20ffcf1001ad37f` |
| `fixtures/noaa/8443970/datums.json` | `https://api.tidesandcurrents.noaa.gov/mdapi/prod/webapi/stations/8443970/datums.json?units=metric` | `1be5e5d22f3e210a51696ad825624e9daac9081010e0a3224992ccbed74e3ae1` |
| `fixtures/noaa/8443970/harcon.json` | `https://api.tidesandcurrents.noaa.gov/mdapi/prod/webapi/stations/8443970/harcon.json?units=metric` | `3a96109c469298b491f744d47a71ff780abcc10786054c6837ac4fd1188a7e42` |
| `fixtures/noaa/8443970/predictions_2026-08-15_2026-08-21.json` | `https://api.tidesandcurrents.noaa.gov/api/prod/datagetter?product=predictions&application=amar&begin_date=20260815&end_date=20260821&datum=MLLW&station=8443970&time_zone=gmt&units=metric&interval=h&format=json` | `9ea15963bfe541e4f132070d29ea5baac8c133e1e7145c539dea68077c84d71f` |
| `fixtures/noaa/8443970/predictions_2031-08-15_2031-08-21.json` | `https://api.tidesandcurrents.noaa.gov/api/prod/datagetter?product=predictions&application=amar&begin_date=20310815&end_date=20310821&datum=MLLW&station=8443970&time_zone=gmt&units=metric&interval=h&format=json` | `7fcf6d36f82e3d188504fe52fdfa4c77815c7696ee9fa0bc038d856e3b30587a` |
| `fixtures/noaa/8443970/predictions_2036-08-15_2036-08-21.json` | `https://api.tidesandcurrents.noaa.gov/api/prod/datagetter?product=predictions&application=amar&begin_date=20360815&end_date=20360821&datum=MLLW&station=8443970&time_zone=gmt&units=metric&interval=h&format=json` | `35f5b9246fa9d583f84a7590fefd1e83d7b44df5a66c96901d7472ddbd05d6f6` |
| `fixtures/noaa/8724580/station.json` | `https://api.tidesandcurrents.noaa.gov/mdapi/prod/webapi/stations/8724580.json?expand=details` | `de096aaa312c92f21808564ccf11ff7a2c074b47b802fb85fbbe5e64ca3a7c75` |
| `fixtures/noaa/8724580/datums.json` | `https://api.tidesandcurrents.noaa.gov/mdapi/prod/webapi/stations/8724580/datums.json?units=metric` | `21aef86e1b68073bf7a50d45cb4587d39f08cd4d27cac1e82c0c10586056bbaf` |
| `fixtures/noaa/8724580/harcon.json` | `https://api.tidesandcurrents.noaa.gov/mdapi/prod/webapi/stations/8724580/harcon.json?units=metric` | `e91a9e0c989e8c3c36ae46b9ad608743f6145dade89a3a1ebe38e7b79849b5b6` |
| `fixtures/noaa/8724580/predictions_2026-08-15_2026-08-21.json` | `https://api.tidesandcurrents.noaa.gov/api/prod/datagetter?product=predictions&application=amar&begin_date=20260815&end_date=20260821&datum=MLLW&station=8724580&time_zone=gmt&units=metric&interval=h&format=json` | `0904c113250d94b9519fbeb4999f00af3cf4cd723d834107dd2c311c42544b35` |
| `fixtures/noaa/8724580/predictions_2031-08-15_2031-08-21.json` | `https://api.tidesandcurrents.noaa.gov/api/prod/datagetter?product=predictions&application=amar&begin_date=20310815&end_date=20310821&datum=MLLW&station=8724580&time_zone=gmt&units=metric&interval=h&format=json` | `873877cf64d2ef858d0375785b71b1aebf9ccf6c56e920be8e7d819e86a378fa` |
| `fixtures/noaa/8724580/predictions_2036-08-15_2036-08-21.json` | `https://api.tidesandcurrents.noaa.gov/api/prod/datagetter?product=predictions&application=amar&begin_date=20360815&end_date=20360821&datum=MLLW&station=8724580&time_zone=gmt&units=metric&interval=h&format=json` | `a7f3f60e8a1f96a5cf2160d3487268317149f2b32f8c29fc9f3e05dff6fee9f3` |
| `fixtures/noaa/8729840/station.json` | `https://api.tidesandcurrents.noaa.gov/mdapi/prod/webapi/stations/8729840.json?expand=details` | `8a17a536bc0138627387c5bb226a7635c177ef40ff34913d133475c76ef70df1` |
| `fixtures/noaa/8729840/datums.json` | `https://api.tidesandcurrents.noaa.gov/mdapi/prod/webapi/stations/8729840/datums.json?units=metric` | `86f1d1fb06ff37b40f1fd716ce470e117a37ae3e369fdef22c9bcb65d313a1d2` |
| `fixtures/noaa/8729840/harcon.json` | `https://api.tidesandcurrents.noaa.gov/mdapi/prod/webapi/stations/8729840/harcon.json?units=metric` | `b928ed4b8fc0ac771c8e4feb985f84dd384c5b475448eaf19f3ce1d7af480bb6` |
| `fixtures/noaa/8729840/predictions_2026-08-15_2026-08-21.json` | `https://api.tidesandcurrents.noaa.gov/api/prod/datagetter?product=predictions&application=amar&begin_date=20260815&end_date=20260821&datum=MLLW&station=8729840&time_zone=gmt&units=metric&interval=h&format=json` | `15b0cd7e41afe823745c48aef92eaad9061e359d01b41deadf70a062ea08fa1a` |
| `fixtures/noaa/8729840/predictions_2031-08-15_2031-08-21.json` | `https://api.tidesandcurrents.noaa.gov/api/prod/datagetter?product=predictions&application=amar&begin_date=20310815&end_date=20310821&datum=MLLW&station=8729840&time_zone=gmt&units=metric&interval=h&format=json` | `cbfe4cb67428321be3cce7378fd1c99fe12dd5ec82aa557ec60439d465cb06cd` |
| `fixtures/noaa/8729840/predictions_2036-08-15_2036-08-21.json` | `https://api.tidesandcurrents.noaa.gov/api/prod/datagetter?product=predictions&application=amar&begin_date=20360815&end_date=20360821&datum=MLLW&station=8729840&time_zone=gmt&units=metric&interval=h&format=json` | `f670d319fe4884e3a8cc478b1693d82aedda3df45f09e74892d3aa00b8d9cc1e` |
| `fixtures/noaa/8771450/station.json` | `https://api.tidesandcurrents.noaa.gov/mdapi/prod/webapi/stations/8771450.json?expand=details` | `933ad49562c44d433b3e427532a7dcc7f49ed0cf3c558b2367358e51e170041c` |
| `fixtures/noaa/8771450/datums.json` | `https://api.tidesandcurrents.noaa.gov/mdapi/prod/webapi/stations/8771450/datums.json?units=metric` | `f375a89c45eda0a0b23f1648bfa01161658730fee03879bad92edcb9f3922fa0` |
| `fixtures/noaa/8771450/harcon.json` | `https://api.tidesandcurrents.noaa.gov/mdapi/prod/webapi/stations/8771450/harcon.json?units=metric` | `1931ecb3560a6f6c70558921b802d34ae31de1187a5d9773baa541e4a99f6636` |
| `fixtures/noaa/8771450/predictions_2026-08-15_2026-08-21.json` | `https://api.tidesandcurrents.noaa.gov/api/prod/datagetter?product=predictions&application=amar&begin_date=20260815&end_date=20260821&datum=MLLW&station=8771450&time_zone=gmt&units=metric&interval=h&format=json` | `01296b264a7e9948e640eeb7c121097c9e19444aa49f5b3ad402636882e212fd` |
| `fixtures/noaa/8771450/predictions_2031-08-15_2031-08-21.json` | `https://api.tidesandcurrents.noaa.gov/api/prod/datagetter?product=predictions&application=amar&begin_date=20310815&end_date=20310821&datum=MLLW&station=8771450&time_zone=gmt&units=metric&interval=h&format=json` | `7a09cfc3424895341c99c967cc5a842f52f23108f584f2f5bd7f3567a01b4846` |
| `fixtures/noaa/8771450/predictions_2036-08-15_2036-08-21.json` | `https://api.tidesandcurrents.noaa.gov/api/prod/datagetter?product=predictions&application=amar&begin_date=20360815&end_date=20360821&datum=MLLW&station=8771450&time_zone=gmt&units=metric&interval=h&format=json` | `3e293e77e20dae4ab3dd44e7c1f2bd0bfdc96f59ad9984976ec6116649199814` |
| `fixtures/noaa/9414290/station.json` | `https://api.tidesandcurrents.noaa.gov/mdapi/prod/webapi/stations/9414290.json?expand=details` | `8926aa535f652f09d4d6dd7d8f8996e8384289bd9e704ec97b5097a19c5887d7` |
| `fixtures/noaa/9414290/datums.json` | `https://api.tidesandcurrents.noaa.gov/mdapi/prod/webapi/stations/9414290/datums.json?units=metric` | `7b5d1b79efb7656499fd91508fec05bce539f1eec177f42456ac6aa4b1b4c79f` |
| `fixtures/noaa/9414290/harcon.json` | `https://api.tidesandcurrents.noaa.gov/mdapi/prod/webapi/stations/9414290/harcon.json?units=metric` | `7129032615c5638aced9f798596e1805809ea6cb0fec29d9b99c7c034d1667eb` |
| `fixtures/noaa/9414290/predictions_2026-08-15_2026-08-21.json` | `https://api.tidesandcurrents.noaa.gov/api/prod/datagetter?product=predictions&application=amar&begin_date=20260815&end_date=20260821&datum=MLLW&station=9414290&time_zone=gmt&units=metric&interval=h&format=json` | `6d6c03af429ef6180c5c763b70bf9a57c9ef6362a5b44b872273e68fdd730bb7` |
| `fixtures/noaa/9414290/predictions_2031-08-15_2031-08-21.json` | `https://api.tidesandcurrents.noaa.gov/api/prod/datagetter?product=predictions&application=amar&begin_date=20310815&end_date=20310821&datum=MLLW&station=9414290&time_zone=gmt&units=metric&interval=h&format=json` | `dd39c9832e046b2c99cfc571e0bc7528a83405645a1e0f0c606e3b1d2f15203c` |
| `fixtures/noaa/9414290/predictions_2036-08-15_2036-08-21.json` | `https://api.tidesandcurrents.noaa.gov/api/prod/datagetter?product=predictions&application=amar&begin_date=20360815&end_date=20360821&datum=MLLW&station=9414290&time_zone=gmt&units=metric&interval=h&format=json` | `72cdb113cbec4380aa6d807362464b1cfe6e981c38ae8fcca2df9a956ce5dc9c` |
| `fixtures/noaa/9447130/station.json` | `https://api.tidesandcurrents.noaa.gov/mdapi/prod/webapi/stations/9447130.json?expand=details` | `d25a07ef8ddf34afc7a1f09ab770371a42142c8c82045879356356092483cdc9` |
| `fixtures/noaa/9447130/datums.json` | `https://api.tidesandcurrents.noaa.gov/mdapi/prod/webapi/stations/9447130/datums.json?units=metric` | `1d8cd5624607384fb64dca8eaad378492147ebb381e1a66e532f1657327b48ba` |
| `fixtures/noaa/9447130/harcon.json` | `https://api.tidesandcurrents.noaa.gov/mdapi/prod/webapi/stations/9447130/harcon.json?units=metric` | `6ade2b0c7e73ecfe86c06ff7c888322c63102ad2e7193da46cf6cbafd65b096a` |
| `fixtures/noaa/9447130/predictions_2026-08-15_2026-08-21.json` | `https://api.tidesandcurrents.noaa.gov/api/prod/datagetter?product=predictions&application=amar&begin_date=20260815&end_date=20260821&datum=MLLW&station=9447130&time_zone=gmt&units=metric&interval=h&format=json` | `d1204b8921f56c5ec44a7e653e267a737f3b751bacc6f99fd29eb99833d47531` |
| `fixtures/noaa/9447130/predictions_2031-08-15_2031-08-21.json` | `https://api.tidesandcurrents.noaa.gov/api/prod/datagetter?product=predictions&application=amar&begin_date=20310815&end_date=20310821&datum=MLLW&station=9447130&time_zone=gmt&units=metric&interval=h&format=json` | `46a8993a55d774decc1f38aea5afe85d4695a90f8d9c9818f48e7c24d4d03e39` |
| `fixtures/noaa/9447130/predictions_2036-08-15_2036-08-21.json` | `https://api.tidesandcurrents.noaa.gov/api/prod/datagetter?product=predictions&application=amar&begin_date=20360815&end_date=20360821&datum=MLLW&station=9447130&time_zone=gmt&units=metric&interval=h&format=json` | `86caf2ae69b2a6eab48dd5830689a93747e20c0ec4acf78d95086fb199625d6c` |

## Fixtures M3 PM/BM

Extraction : 2026-07-06. Produit NOAA `predictions`, `interval=hilo`,
datum `MLLW`, unités métriques, fuseau `gmt`. Les fenêtres 2031 sont limitées
à Seattle, témoin hors réglage, et Eastport, plus grand marnage du pack.

| Fichier | URL d'origine | SHA-256 |
|---|---|---|
| `fixtures/noaa/1612340/hilo_2026-08-15_2026-08-21.json` | `https://api.tidesandcurrents.noaa.gov/api/prod/datagetter?product=predictions&application=amar&begin_date=20260815&end_date=20260821&datum=MLLW&station=1612340&time_zone=gmt&units=metric&interval=hilo&format=json` | `e39f9cdcf364e85b3b34a893e850b2f67ebd5be24e372fdf0f00acc5b9598520` |
| `fixtures/noaa/8410140/hilo_2026-08-15_2026-08-21.json` | `https://api.tidesandcurrents.noaa.gov/api/prod/datagetter?product=predictions&application=amar&begin_date=20260815&end_date=20260821&datum=MLLW&station=8410140&time_zone=gmt&units=metric&interval=hilo&format=json` | `1383ba4be50894ca852270778adc18808952f0c4c6ea805e3752c21ba8264333` |
| `fixtures/noaa/8410140/hilo_2031-08-15_2031-08-21.json` | `https://api.tidesandcurrents.noaa.gov/api/prod/datagetter?product=predictions&application=amar&begin_date=20310815&end_date=20310821&datum=MLLW&station=8410140&time_zone=gmt&units=metric&interval=hilo&format=json` | `7e480b333cbbacd76ec2b296b0dcbf3349f30dd729716390c2990aee9e85b319` |
| `fixtures/noaa/8443970/hilo_2026-08-15_2026-08-21.json` | `https://api.tidesandcurrents.noaa.gov/api/prod/datagetter?product=predictions&application=amar&begin_date=20260815&end_date=20260821&datum=MLLW&station=8443970&time_zone=gmt&units=metric&interval=hilo&format=json` | `dc24b154bfe1fad00538c814ca05099bead242cd6b00447daff7b1217cc918b7` |
| `fixtures/noaa/8724580/hilo_2026-08-15_2026-08-21.json` | `https://api.tidesandcurrents.noaa.gov/api/prod/datagetter?product=predictions&application=amar&begin_date=20260815&end_date=20260821&datum=MLLW&station=8724580&time_zone=gmt&units=metric&interval=hilo&format=json` | `64a2995f3e81c68d1fb67b44a4ebd1bee83a4db894f2ad597295a6642888b021` |
| `fixtures/noaa/8729840/hilo_2026-08-15_2026-08-21.json` | `https://api.tidesandcurrents.noaa.gov/api/prod/datagetter?product=predictions&application=amar&begin_date=20260815&end_date=20260821&datum=MLLW&station=8729840&time_zone=gmt&units=metric&interval=hilo&format=json` | `21025d74dfdddf13220183ff52857d22f8c0c78a6db3327aebbfb038fed5ea4d` |
| `fixtures/noaa/8771450/hilo_2026-08-15_2026-08-21.json` | `https://api.tidesandcurrents.noaa.gov/api/prod/datagetter?product=predictions&application=amar&begin_date=20260815&end_date=20260821&datum=MLLW&station=8771450&time_zone=gmt&units=metric&interval=hilo&format=json` | `143ba9cb67c4efdeecb4ad4eea0ead3c58b6258cd215397b5e893962ce7b6402` |
| `fixtures/noaa/9414290/hilo_2026-08-15_2026-08-21.json` | `https://api.tidesandcurrents.noaa.gov/api/prod/datagetter?product=predictions&application=amar&begin_date=20260815&end_date=20260821&datum=MLLW&station=9414290&time_zone=gmt&units=metric&interval=hilo&format=json` | `2a7d303b5fb4e0f4d5a96fe5c597a84f3fe9bcb31fec2222db94b9f776df7b41` |
| `fixtures/noaa/9447130/hilo_2026-08-15_2026-08-21.json` | `https://api.tidesandcurrents.noaa.gov/api/prod/datagetter?product=predictions&application=amar&begin_date=20260815&end_date=20260821&datum=MLLW&station=9447130&time_zone=gmt&units=metric&interval=hilo&format=json` | `dff6e318b4ad5e9d4d81a6ae5cdca7d9fe9353a8c90959a6bc6c00046cc9778e` |
| `fixtures/noaa/9447130/hilo_2031-08-15_2031-08-21.json` | `https://api.tidesandcurrents.noaa.gov/api/prod/datagetter?product=predictions&application=amar&begin_date=20310815&end_date=20310821&datum=MLLW&station=9447130&time_zone=gmt&units=metric&interval=hilo&format=json` | `a79449cbb1eaaac756fb8a684864661537e8b8b72c53711b9ad905b4ff7763db` |

## Pack M1

| Fichier | Origine | SHA-256 |
|---|---|---|
| `data/packs/noaa_m0.json` | `cargo run -p amar -- pack-noaa --fixtures fixtures/noaa --out data/packs/noaa_m0.json --extracted-at 2026-07-06 --station 8410140 --station 8443970 --station 9414290 --station 8729840 --station 9447130 --station 1612340 --station 8724580 --station 8771450` | `8965539b65cd8bc75ef96f24adfd056a1d5147c1fdae9e67ff40a0eb1f214594` |

## REFMAR Brest expérimental

Les observations de Brest proviennent du service REFMAR/data.shom.fr, avec
attribution `Shom / REFMAR`, sous Licence Ouverte 2.0 Etalab. Le produit retenu
est `sources=4`, données horaires validées, en mètres, fuseau UTC, référence
verticale `zero_hydrographique`.

Station : `shom_id=3`, `BREST`, réseau `RONIM`.

Référence verticale issue de la fiche `completetidegauge/3` : RAM id `Brest`,
ZH = -3.635 m par rapport à `IGN69`.

| Fichier | Origine | SHA-256 |
|---|---|---|
| `fixtures/refmar/brest_tidegauge.json` | `https://services.data.shom.fr/maregraphie/service/completetidegauge/3` | `5327d51c4e210f656b1ee2b3ff0d6e1b5bbd0a8a8cddfcb1c13c556d43e994cf` |
| `fixtures/refmar/brest_validated_hourly_2025-01-01_2026-07-01.csv` | `https://services.data.shom.fr/maregraphie/observation/json/3?sources=4`, fenêtres de 31 jours | `e3c63f5cea16eeb0a55ac24ca2c68aca6221b2908e9e5236c455b64911e9373d` |
| `fixtures/refmar/brest_validated_hourly_2021-01-01_2026-07-01.csv` | `https://services.data.shom.fr/maregraphie/observation/json/3?sources=4`, fenêtres de 31 jours | `17951bab11dc99220b8462da118df6ba4412a2492eb36a8c01b5935b6cd5a8e2` |
| `fixtures/refmar/benchmark_brest_v1.json` | Fenêtre hors calibration `2026-04-01T00:00:00Z/2026-07-01T00:00:00Z` | `d36f445c320c17ba323fbe572e0cb93d45eba846aeff0260ee9d1b3631a6bf6f` |
| `data/packs/amar-data-brest-experimental.json` | `cargo run -p amar-calibrate -- build-brest-pack` | `e377e25754e9cbc6a05e732d0dcff2db2c1305b57037432312015ae45d749919` |

Le checksum interne `benchmark_brest_v1.checksum_sha256` couvre le masque
horaire et les observations de la fenêtre de validation :
`531da284f68bb9acf77c9d21b90e0fd3d787809c0ddb0cd4d118c63eddc0ac42`.

Les constantes Brest publiées par amar sont dérivées des observations REFMAR ;
elles ne sont pas des constantes SHOM et ne doivent pas être présentées comme
équivalentes.

## REFMAR France v0.4 expérimental

Le pack France v0.4 utilise la même source et la même licence que Brest :
service REFMAR/data.shom.fr, attribution `Shom / REFMAR`, Licence Ouverte 2.0
Etalab, produit `sources=4` données horaires validées, en mètres, UTC,
référence verticale `zero_hydrographique` locale.

Les observations longues de calibration ne sont pas commitées. Les manifestes
ci-dessous publient le SHA-256 des CSV d'entrée produits par re-fetch via
`amar-calibrate calibrate-france`; les benchmarks commités couvrent seulement
la fenêtre de validation de trois mois.

| Fichier | Origine | SHA-256 |
|---|---|---|
| `data/packs/amar-data-france-experimental.json` | `cargo run -p amar-calibrate -- calibrate-france` | `9aa5249920396be1fa91cdd1c5ec58301dc5e3c074230290c0a98f83591830b2` |

| Station | Observations SHA-256 | Manifeste | Benchmark |
|---|---|---|---|
| `refmar:111` Boulogne-sur-Mer | `16b2ff8809ce806026fcfcb6e4bdf0ca0cefb472f8e07c31dcfd73c47fad5616` | `fixtures/refmar/manifests/boulogne_sur_mer_observations.json` | `fixtures/refmar/benchmarks/benchmark_boulogne_sur_mer_v1.json` |
| `refmar:160` Concarneau | `c8b4daaf998278bfaf08260e436309bdaa08d82e5fe8ce6ff3a2969b00b1b8a1` | `fixtures/refmar/manifests/concarneau_observations.json` | `fixtures/refmar/benchmarks/benchmark_concarneau_v1.json` |
| `refmar:24` Dieppe | `fc64ac304b35a4fc921458d3a247f6d463de527217c7f4c27f6d20823ceeb22b` | `fixtures/refmar/manifests/dieppe_observations.json` | `fixtures/refmar/benchmarks/benchmark_dieppe_v1.json` |
| `refmar:2` Dunkerque | `f8bb2eef957a1371fa1ffa89f1a2f58fedf7743b7aa86f620d13ad88e0d7d870` | `fixtures/refmar/manifests/dunkerque_observations.json` | `fixtures/refmar/benchmarks/benchmark_dunkerque_v1.json` |
| `refmar:34` La Rochelle-Pallice | `4d449b0b21fb1410cd7754e64eed11a5dd7da9e62deafb3fc737ade9f087021d` | `fixtures/refmar/manifests/la_rochelle_pallice_observations.json` | `fixtures/refmar/benchmarks/benchmark_la_rochelle_pallice_v1.json` |
| `refmar:152` Le Conquet | `70d5d2e01a8172743286ed5090f5ed99f43ed2f0e18eec6d0c8e7e9c0e474d52` | `fixtures/refmar/manifests/le_conquet_observations.json` | `fixtures/refmar/benchmarks/benchmark_le_conquet_v1.json` |
| `refmar:4` Le Havre | `48222b92a9c0a93fa4393d10081580d1edac0b1a92ff09baf07d3ca79f5d38d2` | `fixtures/refmar/manifests/le_havre_observations.json` | `fixtures/refmar/benchmarks/benchmark_le_havre_v1.json` |
| `refmar:311` Ouistreham | `c733522556ae7c3fab0d46a9250b5de25190cb863981f854032ccd000d3ae109` | `fixtures/refmar/manifests/ouistreham_observations.json` | `fixtures/refmar/benchmarks/benchmark_ouistreham_v1.json` |
| `refmar:54` Roscoff | `232e848b1c58234c59d60f847c8a3f1ba0c073f87c5273af3933447522c7d339` | `fixtures/refmar/manifests/roscoff_observations.json` | `fixtures/refmar/benchmarks/benchmark_roscoff_v1.json` |
| `refmar:410` Saint-Malo | `e124beecbd91efbc2e9a68835986fb87d4d981e304d2f9e2a537d5f226b6fe94` | `fixtures/refmar/manifests/saint_malo_observations.json` | `fixtures/refmar/benchmarks/benchmark_saint_malo_v1.json` |
| `refmar:37` Saint-Nazaire | `2ccbb3b36fb174aff5222fddc7107b4eca66cf286ba29c388c97bdeec19f747d` | `fixtures/refmar/manifests/saint_nazaire_observations.json` | `fixtures/refmar/benchmarks/benchmark_saint_nazaire_v1.json` |

## Open-Meteo diagnostic Brest M2.2

La pression horaire de surface utilisée pour le diagnostic baromètre inverse
provient de l'API historique Open-Meteo, variable `surface_pressure`, timezone
`GMT`, `cell_selection=nearest`, coordonnées Brest
`48.38290024,-4.49503994`. Elle n'entre ni dans `/tide`, ni dans le pack, ni
dans `benchmark_brest_v1`.

La page licence Open-Meteo indique que les données API sont proposées sous
Attribution 4.0 International (CC BY 4.0), avec attribution à Open-Meteo et
lien vers la licence. La page de l'API historique documente `surface_pressure`
en hPa et les sources de réanalyse.

| Fichier | Origine | SHA-256 |
|---|---|---|
| `fixtures/open_meteo/brest_surface_pressure_2026-04-01_2026-06-30.json` | `https://archive-api.open-meteo.com/v1/archive?latitude=48.38290024&longitude=-4.49503994&start_date=2026-04-01&end_date=2026-06-30&hourly=surface_pressure&timezone=GMT&cell_selection=nearest` | `24c254012f89c5510d4e63b6d00c187b97233c327b2bd146d5082de22bfcd655` |
