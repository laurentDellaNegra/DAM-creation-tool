# Glossary of map-flag tokens

| Token        | Type          | Meaning                                                         |
| ------------ | ------------- | --------------------------------------------------------------- |
| `LEVEL`      | prefix        | Marks the start of a LEVEL-format remarks string                |
| `ll`         | key=value     | Lower level (FL or feet)                                        |
| `ul`         | key=value     | Upper level (FL or feet)                                        |
| `vbuff`      | key=value     | Vertical buffer (value × 100 = feet, e.g. `vbuff=20` = 2000 ft) |
| `TEXT`       | key=value     | Display text / label                                            |
| `ft`         | unit          | Feet (level value is in feet, not FL)                           |
| `lft`        | flag          | Lower-level feet checkbox                                       |
| `uft`        | flag          | Upper-level feet checkbox                                       |
| `flc`        | flag          | FL Correction: use A9-defined value                             |
| `qnh`        | flag          | QNH altitude reference                                          |
| `bt`         | flag          | Begin time (Start Time)                                         |
| `et`         | flag          | End time                                                        |
| `dl`         | flag          | Display Levels checkbox                                         |
| `lnbu`       | buffer        | Lower-level no-buffer                                           |
| `unbu`       | buffer        | Upper-level no-buffer                                           |
| `uhbu`       | buffer        | Upper-level half-buffer                                         |
| `lhbu`       | buffer        | Lower-level half-buffer (does **not** appear in any GVA map)    |
| `restricted` | airspace type | Restricted area                                                 |
| `danger`     | airspace type | Danger area                                                     |
| `tra`        | airspace type | Temporary Reserved Airspace                                     |
| `glider`     | airspace type | Glider area                                                     |
| `parachute`  | airspace type | Parachute area                                                  |
| `other`      | airspace type | Other airspace type                                             |
| `APP`        | station       | Approach                                                        |
| `ARF`        | station       | Area Flight Information (ARFA)                                  |
| `BRN`        | station       | Bern                                                            |
| `BUO`        | station       | Buochs                                                          |
| `DUB`        | station       | Dübendorf                                                       |
| `EMM`        | station       | Emmen                                                           |
| `FIC`        | station       | Flight Information Centre                                       |
| `LAC`        | station       | Lower Area Control                                              |
| `LUG`        | station       | Lugano                                                          |
| `MEZ`        | station       | MEZ                                                             |
| `SIO`        | station       | Sion                                                            |
| `STG`        | station       | St. Gallen                                                      |
| `TWR`        | station       | Tower                                                           |
| `UAC`        | station       | Upper Area Control                                              |
