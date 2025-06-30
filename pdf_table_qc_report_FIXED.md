# 📊 PDF Table Extraction - Quality Control Report

**📄 Source:** `EXAMPLE_NIGHTMARE_PDF.pdf`  
**📋 Tables Found:** 23  
**🕒 Generated:** Just now  

## 🚨 Critical Document Understanding Issue

> **Root Problem:** The extractor processes tables without understanding environmental lab document conventions.

### 📖 Missing Document Context:

1. **Data Qualifiers Convention:**
   - `U` = Undetected (below detection limit)
   - `J` = Estimated value (detected but below reporting limit)
   - These should ALWAYS be in separate columns from numeric values

2. **Expected Column Structure Pattern:**
   - `Concentration | Qualifier | Reporting Limit | Method Detection Limit`
   - This pattern repeats for each analyte group

3. **Document Legend/Header:**
   - Document likely contains qualifier definitions
   - Pre-scanning could inform extraction logic

### 🔧 Suggested Extraction Improvements:

1. **Document-aware preprocessing:** Scan for data conventions and legends
2. **Pattern recognition:** Detect repeating column structures (Conc/Q/RL/MDL)
3. **Context integration:** Use document metadata to guide table interpretation
4. **Qualifier separation:** Automatically split combined values like '0.046 U'

---

## 📈 Extraction Quality Summary

🔴 **39 likely misplaced qualifiers detected across all tables**

⚠️ **110 standalone qualifiers found** - verify column placement

---


## 📋 Table 1 - Environmental Lab Data

> **⚠️ Document Convention Analysis:**
> - **U** = Undetected (below detection limit) - should be in separate qualifier column
> - **J** = Estimated value (detected but below reporting limit) - should be in separate qualifier column
> - **Expected Pattern:** Concentration | Qualifier | Reporting Limit | Method Detection Limit
> - **Look for:** Values like '0.046 U' that should be split into '0.046' and 'U'

| Representative  Pennsylvania Soil  MSC (lower of Soil  to GW and Direct  Contact Screening  Values) |  Pennsylvania  Non-Residential  Soil Statewide  Health Standard  Vapor Intrusion  Screening Values  |                       Sample ID: Area: Laboratory ID: 410-57140-5 9/23/2021                       | Sample ID: Area: Laboratory ID: 410-57140-5 9/23/2021 | Sample ID: Area: Laboratory ID: 410-57140-5 9/23/2021 | Sample ID: Area: Laboratory ID: 410-57140-5 9/23/2021 | 7551-South 1 410-57140-7 9/23/2021 | 7551-South 1 410-57140-7 9/23/2021 | 7551-South 1 410-57140-7 9/23/2021 | 7551-South 1 410-57140-7 9/23/2021 | DUP-9 1 410-56522-16 9/23/2021 | DUP-9 1 410-56522-16 9/23/2021 | DUP-9 1 410-56522-16 9/23/2021 | UNK-ST-S 1 410-50503-6 8/6/2021 | UNK-ST-S 1 410-50503-6 8/6/2021 | UNK-ST-S 1 410-50503-6 8/6/2021 | UNK-ST-N 1 410-50503-7 8/6/2021 | UNK-ST-N 1 410-50503-7 8/6/2021 | UNK-ST-N 1 410-50503-7 8/6/2021 |
|:---------------------------------------------------------------------------------------------------:|:---------------------------------------------------------------------------------------------------:|:-------------------------------------------------------------------------------------------------:|:-----------------------------------------------------:|:-----------------------------------------------------:|:-----------------------------------------------------:|:----------------------------------:|:----------------------------------:|:----------------------------------:|:----------------------------------:|:------------------------------:|:------------------------------:|:------------------------------:|:-------------------------------:|:-------------------------------:|:-------------------------------:|:-------------------------------:|:-------------------------------:|:-------------------------------:|
|                                              ⚠️ **Q**                                               | Representative  Pennsylvania Soil  MSC (lower of Soil  to GW and Direct  Contact Screening  Values) | Pennsylvania  Non-Residential  Soil Statewide  Health Standard  Vapor Intrusion  Screening Values |                         Conc                          |                         Q RL                          |                          MDL                          |               Conc Q               |                 RL                 |                MDL                 |                Conc                |            ⚠️ **Q**            |               RL               |              MDL               |             Conc Q              |               RL                |               MDL               |              Conc               |               RL                |               MDL               | ❌ *EMPTY*  |
|                                     Volatile Organic Compounds                                      |                                     Volatile Organic Compounds                                      |                                    Volatile Organic Compounds                                     |              Volatile Organic Compounds               |              Volatile Organic Compounds               |              Volatile Organic Compounds               |     Volatile Organic Compounds     |     Volatile Organic Compounds     |     Volatile Organic Compounds     |     Volatile Organic Compounds     |   Volatile Organic Compounds   |   Volatile Organic Compounds   |   Volatile Organic Compounds   |   Volatile Organic Compounds    |            ❌ *EMPTY*            |            ❌ *EMPTY*            |            ❌ *EMPTY*            |            ❌ *EMPTY*            |            ❌ *EMPTY*            | ❌ *EMPTY*  |
|                                           Ethylbenzene J                                            |                                                 70                                                  |                                                46                                                 |                         0.046                         |                       ⚠️ **U**                        |                         0.58                          |               0.046                |         🔴 **0.00043 U** 🔴          |               0.0054               |              0.00043               |            0.00066             |            ⚠️ **U**            |             0.0082             |             0.00066             |       🔴 **0.000851 J** 🔴        |             0.0079              |             0.00063             |             0.3101              |              0.43               |   0.034    |
|                                        1,2-Dichloroethane U                                         |                                                 0.5                                                 |                                                3.9                                                |                         0.07                          |                       ⚠️ **U**                        |                         0.58                          |                0.07                |         🔴 **0.00065 U** 🔴          |               0.0054               |              0.00065               |            0.00098             |            ⚠️ **U**            |         0.0082 0.00098         |        🔴 **0.00094 U** 🔴        |             0.0079              |             0.00094             |              0.051              |              0.43               |              0.051              | ❌ *EMPTY*  |
|                                       1,3,5-Trimethylbenzene                                        |                                                 93                                                  |                                                93                                                 |                        0.0811                         |                       ⚠️ **J**                        |                         0.58                          |               0.058                |         🔴 **0.00054 U** 🔴          |               0.0054               |              0.00054               |            0.00082             |            ⚠️ **U**            |             0.0082             |             0.00082             |        🔴 **0.00661 J** 🔴        |             0.0079              |              0.001              |              1.10               |              0.43               |   0.043    |
|                                               Toluene                                               |                                                 100                                                 |                                                44                                                 |                         0.07                          |                       ⚠️ **U**                        |                         0.58                          |                0.07                |         🔴 **0.00065 U** 🔴          |               0.0054               |              0.00065               |            0.00111             |            ⚠️ **J**            |             0.0082             |             0.00098             |        🔴 **0.00281 J** 🔴        |             0.0079              |             0.00094             |              0.740              |              0.43               |  0.05100   |
|                                           Xylenes, total                                            |                                                1,000                                                |                                                990                                                |                         0.16                          |                       ⚠️ **U**                        |                         1.200                         |                0.16                |          🔴 **0.0015 U** 🔴          |               0.011                |               0.0015               |             0.0023             |            ⚠️ **U**            |             0.016              |             0.0023              |        🔴 **0.00721 J** 🔴        |              0.016              |              0.002              |              2.40               |              0.860              |   0.120    |
|                                      Methyl tert-butyl ether U                                      |                                                 2.0                                                 |                                                1.4                                                |                        0.2401                         |                       ⚠️ **J**                        |                         0.58                          |               0.058                |         🔴 **0.00054 U** 🔴          |               0.0054               |              0.00054               |             0.023              |             0.0082             |            0.00082             |        🔴 **0.00181 J** 🔴        |             0.0079              |             0.00079             |              0.043              |              0.43               |              0.043              | ❌ *EMPTY*  |
|                                               Benzene                                               |                                                 0.5                                                 |                                               0.13                                                |                         0.058                         |                       ⚠️ **U**                        |                         0.58                          |               0.058                |         🔴 **0.00054 U** 🔴          |               0.0054               |              0.00054               |            0.00121             |            ⚠️ **J**            |             0.0082             |             0.00082             |        🔴 **0.00181 J** 🔴        |             0.0079              |             0.00079             |              0.560              |              0.43               |   0.043    |
|                                             Naphthalene                                             |                                                 25                                                  |                                                25                                                 |                         0.23                          |                       ⚠️ **U**                        |                         0.58                          |            0.23 0.0022             |              ⚠️ **U**              |               0.0054               |               0.0022               |             0.0033             |            ⚠️ **U**            |             0.0082             |             0.0033              |        🔴 **0.00331 J** 🔴        |             0.0079              |             0.00310             |              3.40               |              0.43               |  0.17000   |
|                                       1,2,4-Trimethylbenzene                                        |                                                 300                                                 |                                                300                                                |                        0.1701                         |                       ⚠️ **J**                        |                         0.58                          |           0.058 0.00054            |              ⚠️ **U**              |               0.0054               |              0.00054               |            0.00082             |            ⚠️ **U**            |             0.0082             |          0.00082 0.012          |             0.0079              |             0.00079             |              2.80               |              0.43               |              0.043              | ❌ *EMPTY*  |
|                                         Isopropylbenzene J                                          |                                                2,500                                                |                                               2,500                                               |                         0.046                         |                       ⚠️ **U**                        |                         0.58                          |           0.046 0.00043            |              ⚠️ **U**              |               0.0054               |              0.00043               |            0.00066             |            U 0.0082            |            0.00066             |        🔴 **0.00063 U** 🔴        |             0.0079              |             0.00063             |             0.1201              |              0.43               |              0.034              | ❌ *EMPTY*  |
|                                        1,2,-Dibromoethane U                                         |                                                0.005                                                |                                              0.0013                                               |                         0.046                         |                       ⚠️ **U**                        |                         0.58                          |           0.046 0.00043            |              ⚠️ **U**              |               0.0054               |              0.00043               |            0.00066             |            U 0.0082            |            0.00066             |        🔴 **0.00063 U** 🔴        |             0.0079              |             0.00063             |              0.034              |              0.43               |              0.034              | ❌ *EMPTY*  |
|                                   Semivolatile Organic Compounds                                    |                                              ❌ *EMPTY*                                              |                                             ❌ *EMPTY*                                             |                       ❌ *EMPTY*                       |                       ❌ *EMPTY*                       |                       ❌ *EMPTY*                       |             ❌ *EMPTY*              |             ❌ *EMPTY*              |             ❌ *EMPTY*              |             ❌ *EMPTY*              |           ❌ *EMPTY*            |           ❌ *EMPTY*            |           ❌ *EMPTY*            |            ❌ *EMPTY*            |            ❌ *EMPTY*            |            ❌ *EMPTY*            |            ❌ *EMPTY*            |            ❌ *EMPTY*            |            ❌ *EMPTY*            | ❌ *EMPTY*  |
|                                             Anthracene                                              |                                                 350                                                 |                                                --                                                 |                         0.63                          |                         0.019                         |                         0.004                         |               0.059                |               0.022                |               0.004                |               0.0221               |             0.028              |             0.006              |        🔴 **0.0241 J** 🔴        |              0.026              |              0.005              |               11                |              0.210              |              0.042              |            ❌ *EMPTY*            | ❌ *EMPTY*  |
|                                         Benzo(a)anthracene                                          |                                                 130                                                 |                                                --                                                 |                          0.5                          |                         0.019                         |                         0.004                         |                0.19                |               0.022                |               0.004                |               0.054                |             0.028              |             0.006              |        🔴 **0.0251 J** 🔴        |              0.026              |              0.005              |              8.70               |              0.210              |              0.042              |            ❌ *EMPTY*            | ❌ *EMPTY*  |
|                                           Benzo(a)pyrene                                            |                                                 46                                                  |                                                --                                                 |                         0.27                          |                         0.019                         |                         0.004                         |                0.16                |               0.022                |               0.004                |               0.058                |             0.028              |             0.006              |        🔴 **0.0151 J** 🔴        |              0.026              |              0.005              |              5.30               |              0.210              |              0.042              |            ❌ *EMPTY*            | ❌ *EMPTY*  |
|                                        Benzo(b)fluoranthene                                         |                                                 76                                                  |                                                --                                                 |                         0.43                          |                         0.019                         |                         0.004                         |                0.19                |               0.022                |               0.004                |               0.069                |             0.028              |             0.006              |        🔴 **0.0231 J** 🔴        |              0.026              |              0.005              |              6.80               |              0.210              |              0.042              |            ❌ *EMPTY*            | ❌ *EMPTY*  |
|                                        Benzo(g,h,i)perylene                                         |                                                 180                                                 |                                                --                                                 |                         0.18                          |                         0.019                         |                         0.004                         |                0.11                |               0.022                |               0.004                |               0.062                |             0.028              |             0.006              |        🔴 **0.0052 U** 🔴        |              0.026              |              0.005              |              2.70               |              0.021              |              0.004              |            ❌ *EMPTY*            | ❌ *EMPTY*  |
|                                              Chrysene                                               |                                                 230                                                 |                                                --                                                 |                         0.53                          |                         0.019                         |                         0.004                         |                0.16                |               0.022                |               0.004                |               0.068                |             0.028              |             0.006              |        🔴 **0.0221 J** 🔴        |              0.026              |              0.005              |              8.00               |              0.210              |              0.042              |            ❌ *EMPTY*            | ❌ *EMPTY*  |
|                                              Fluorene                                               |                                                3,800                                                |                                                --                                                 |                         0.78                          |                         0.019                         |                     0.004 0.0171                      |              ⚠️ **J**              |               0.022                |               0.004                |              0.00911               |             0.028              |             0.006              |        🔴 **0.0052 U** 🔴        |              0.026              |              0.005              |              3.80               |              0.021              |              0.004              |            ❌ *EMPTY*            | ❌ *EMPTY*  |
|                                            Phenanthrene                                             |                                               10,000                                                |                                                --                                                 |                           4                           |                         0.019                         |                         0.005                         |                0.19                |               0.022                |               0.005                |               0.091                |             0.028              |             0.007              |             0.200              |              0.026              |              0.006              |               29                |              0.210              |              0.051              |            ❌ *EMPTY*            | ❌ *EMPTY*  |
|                                               Pyrene                                                |                                                2,200                                                |                                                --                                                 |                          1.8                          |                         0.019                         |                         0.004                         |                0.32                |               0.022                |               0.004                |               0.099                |             0.028              |             0.006              |             0.098              |              0.026              |              0.005              |               15                |              0.210              |              0.042              |            ❌ *EMPTY*            | ❌ *EMPTY*  |
|                                             Inorganics                                              |                                              ❌ *EMPTY*                                              |                                             ❌ *EMPTY*                                             |                       ❌ *EMPTY*                       |                       ❌ *EMPTY*                       |                       ❌ *EMPTY*                       |             ❌ *EMPTY*              |             ❌ *EMPTY*              |             ❌ *EMPTY*              |             ❌ *EMPTY*              |           ❌ *EMPTY*            |           ❌ *EMPTY*            |           ❌ *EMPTY*            |            ❌ *EMPTY*            |            ❌ *EMPTY*            |            ❌ *EMPTY*            |            ❌ *EMPTY*            |            ❌ *EMPTY*            |            ❌ *EMPTY*            | ❌ *EMPTY*  |
|                                               Lead 32                                               |                                                 450                                                 |                                                --                                                 |                          36                           |                         1.600                         |                         0.660                         |               2.000                |               0.790                |                 42                 |               2.000                |             0.780              |               39               |             2.200              |              0.870              |               65                |              1.600              |              0.640              |            ❌ *EMPTY*            |            ❌ *EMPTY*            | ❌ *EMPTY*  |

### 🔍 Pattern Analysis for this Table:

**❌ 24 misplaced qualifiers found!**
- 🔴 **Misplaced qualifier detected:** `0.00043 U` (should be split)
- 🔴 **Misplaced qualifier detected:** `0.000851 J` (should be split)
- 🔴 **Misplaced qualifier detected:** `0.00065 U` (should be split)
- 🔴 **Misplaced qualifier detected:** `0.00094 U` (should be split)
- 🔴 **Misplaced qualifier detected:** `0.00054 U` (should be split)
- ... and 19 more
⚠️ **26 standalone qualifiers found** (verify they're in correct columns)

---


## 📋 Table 2 - Environmental Lab Data

> **⚠️ Document Convention Analysis:**
> - **U** = Undetected (below detection limit) - should be in separate qualifier column
> - **J** = Estimated value (detected but below reporting limit) - should be in separate qualifier column
> - **Expected Pattern:** Concentration | Qualifier | Reporting Limit | Method Detection Limit
> - **Look for:** Values like '0.046 U' that should be split into '0.046' and 'U'

|          Analyte           | Pennsylvania  Non-Residential  Used Aquifer  Groundwater  MSCs | Sample ID:  Laboratory ID:  Collection Date:  Sample Matrix: | WATER 10/25/2021 L2158499-03 MW-003 | WATER 10/25/2021 L2158499-03 MW-003 | WATER 10/25/2021 L2158499-03 MW-003 | WATER 10/25/2021 L2158499-03 MW-003 | WATER 10/25/2021 L2158499-04 MW-004 | WATER 10/25/2021 L2158499-04 MW-004 | WATER 10/25/2021 L2158499-04 MW-004 | WATER 10/25/2021 L2158499-04 MW-004 |
|:--------------------------:|:--------------------------------------------------------------:|:------------------------------------------------------------:|:-----------------------------------:|:-----------------------------------:|:-----------------------------------:|:-----------------------------------:|:-----------------------------------:|:-----------------------------------:|:-----------------------------------:|:-----------------------------------:|
|            Conc            |                            ⚠️ **Q**                            |                              RL                              |                 MDL                 |                Conc                 |              ⚠️ **Q**               |                 RL                  |                 MDL                 |              ❌ *EMPTY*              |              ❌ *EMPTY*              |              ❌ *EMPTY*              |
|    Benzo(k)fluoranthene    |                              0.55                              |                             0.01                             |              ⚠️ **U**               |                 0.1                 |                0.01                 |                0.01                 |              ⚠️ **U**               |                 0.1                 |                0.01                 |              ❌ *EMPTY*              |
|  Bis(2-chloroethyl)ether   |                              0.76                              |                             0.02                             |              ⚠️ **U**               |                 0.1                 |                0.02                 |                0.02                 |              ⚠️ **U**               |                 0.1                 |                0.02                 |              ❌ *EMPTY*              |
| Bis(2-ethylhexyl)phthalate |                               6                                |                             0.51                             |              ⚠️ **U**               |                  1                  |                0.51                 |                0.51                 |              ⚠️ **U**               |                  1                  |                0.51                 |              ❌ *EMPTY*              |
|   Dibenzo(a,h)anthracene   |                              0.6                               |                             0.01                             |              ⚠️ **U**               |                0.05                 |                0.01                 |                0.01                 |              ⚠️ **U**               |                0.05                 |                0.01                 |              ❌ *EMPTY*              |
|     Hexachlorobenzene      |                               1                                |                             0.01                             |              ⚠️ **U**               |                0.02                 |                0.01                 |                0.01                 |              ⚠️ **U**               |                0.02                 |                0.01                 |              ❌ *EMPTY*              |
|      Hexachloroethane      |                               1                                |                             0.06                             |              ⚠️ **U**               |                 0.2                 |                0.06                 |                0.06                 |              ⚠️ **U**               |                 0.2                 |                0.06                 |              ❌ *EMPTY*              |
|   Indeno(1,2,3-cd)pyrene   |                              2.3                               |                             0.01                             |              ⚠️ **U**               |                 0.1                 |                0.01                 |                0.01                 |              ⚠️ **U**               |                 0.1                 |                0.01                 |              ❌ *EMPTY*              |
| n-Nitrosodi-n-propylamine  |                               --                               |                             0.01                             |              ⚠️ **U**               |                 0.1                 |                0.01                 |                0.01                 |              ⚠️ **U**               |                 0.1                 |                0.01                 |              ❌ *EMPTY*              |
|     Pentachlorophenol      |                               1                                |                             0.01                             |              ⚠️ **U**               |                 0.1                 |                0.01                 |                0.01                 |              ⚠️ **U**               |                 0.1                 |                0.01                 |              ❌ *EMPTY*              |

### 🔍 Pattern Analysis for this Table:

✅ **No obviously misplaced qualifiers detected**
⚠️ **20 standalone qualifiers found** (verify they're in correct columns)

---


## 📋 Table 3 - Environmental Lab Data

> **⚠️ Document Convention Analysis:**
> - **U** = Undetected (below detection limit) - should be in separate qualifier column
> - **J** = Estimated value (detected but below reporting limit) - should be in separate qualifier column
> - **Expected Pattern:** Concentration | Qualifier | Reporting Limit | Method Detection Limit
> - **Look for:** Values like '0.046 U' that should be split into '0.046' and 'U'

|                                      Analyte:                                      | ( µ g/l) Pennsylvania  Fish and  Aquatic Life  Criteria  | 4/29/2024 L2423395-01 SW-1 | Water 7/22/2024 DEP51-W1 Water | L2441149-13 Water 7/22/2024 L2441149-14 DEP51-W2 | 7/22/2024 L2441149-15 DEP51-W3 | Water 7/22/2024 L2441149-16 DEP51-W4 | Conc Q RL MDL Conc Q RL DEP51-W5-240813 L2445674-12 8/13/2024 Water Water 7/22/2024 L2441149-17 DUP-W (DEP51-W4) |  MDL Conc  | Q RL MDL Water 8/13/2024 L2445674-13 DEP51-W6-240813 |   Water    |
|:----------------------------------------------------------------------------------:|:--------------------------------------------------------:|:--------------------------:|:------------------------------:|:------------------------------------------------:|:------------------------------:|:------------------------------------:|:----------------------------------------------------------------------------------------------------------------:|:----------:|:----------------------------------------------------:|:----------:|
| Conc Q RL MDL Conc Q RL MDL Conc Q RL MDL Conc Q RL MDL Volatile Organics by GC/MS |                      Conc Q RL MDL                       |         ❌ *EMPTY*          |           ❌ *EMPTY*            |                    ❌ *EMPTY*                     |           ❌ *EMPTY*            |              ❌ *EMPTY*               |                                                    ❌ *EMPTY*                                                     | ❌ *EMPTY*  |                      ❌ *EMPTY*                       | ❌ *EMPTY*  |   ❌ *EMPTY*   |  ❌ *EMPTY*  |   ❌ *EMPTY*   |  ❌ *EMPTY*  |  ❌ *EMPTY*  |  ❌ *EMPTY*  |     ❌ *EMPTY*     |    ❌ *EMPTY*    |  ❌ *EMPTY*  |  ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  |  ❌ *EMPTY*   | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  |      ❌ *EMPTY*      | ❌ *EMPTY*  | ❌ *EMPTY*  |
|                                1,2-Dichlorobenzene                                 |                           160                            |            0.18            |             U 2.5              |                       0.18                       |              0.18              |                U 2.5                 |                                                       0.18                                                       |    0.18    |                        U 2.5                         |  2.5 0.18  |  0.18 U 2.5   |   0.18 -    |       -       |      -      |      -      |     - -     |         -         |       2.5       |  ❌ *EMPTY*  |  ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  |  ❌ *EMPTY*   | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  |      ❌ *EMPTY*      | ❌ *EMPTY*  | ❌ *EMPTY*  |
|                                 1,2-Dichloroethane                                 |                           9.9                            |            0.13            |             U 0.5              |                       0.13                       |              0.13              |                U 0.5                 |                                                       0.13                                                       |    0.13    |                        U 0.5                         | 0.18 0.13  |   0.18 0.13   |     U U     |   0.18 0.13   | 0.18 U 0.13 |  ⚠️ **U**   |  0.5 0.13   |    0.13 U 0.5     |      0.13       |      -      |    - - -    |     -      |     -      |     -      |      -       |     -      |    0.5     | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  |      ❌ *EMPTY*      | ❌ *EMPTY*  | ❌ *EMPTY*  |
|                                1,4-Dichlorobenzene                                 |                           150                            |            0.19            |             U 2.5              |                       0.19                       |              0.19              |               ⚠️ **U**               |                                                       2.5                                                        |    0.19    |                         0.19                         |   U 2.5    |     0.19      |    0.19     |     U 2.5     |    0.19     |    0.19     |  ⚠️ **U**   |        2.5        | 0.19 0.19 U 2.5 |    0.19     |    - - -    |     -      |     -      |     -      |      -       |     -      | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  |      ❌ *EMPTY*      | ❌ *EMPTY*  | ❌ *EMPTY*  |
|                                     2-Butanone                                     |                            --                            |            1.9             |              U 5               |                       1.9                        |              1.9               |                 U 5                  |                                                       1.9                                                        |    1.9     |                         U 5                          |    1.9     |      1.9      |  ⚠️ **U**   |     5 1.9     |     1.9     |  ⚠️ **U**   |      5      |     1.9 1.9 U     |     5 1.9 -     |     - -     |      -      |     -      |     -      |     -      |      -       | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  |      ❌ *EMPTY*      | ❌ *EMPTY*  | ❌ *EMPTY*  |
|                                      Acetone                                       |                           3500                           |            8.6             |               5                |                       1.5                        |              1.8               |                 J 5                  |                                                       1.5                                                        |    1.7     |                         J 5                          |    1.5     |      1.5      |     U 5     |      1.5      |     1.6     |  ⚠️ **J**   |    5 1.5    |   🔴 **1.7 J** 🔴   |      5 1.5      |      -      |     - -     |     -      |    - -     |     -      |      -       | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  |      ❌ *EMPTY*      | ❌ *EMPTY*  | ❌ *EMPTY*  |
|                                      Benzene                                       |                           0.58                           |            0.16            |             U 0.5              |                       0.16                       |              0.16              |                U 0.5                 |                                                       0.16                                                       |    0.16    |                        U 0.5                         |    0.16    |     0.16      |    U 0.5    |     0.16      |    0.16     |  ⚠️ **U**   |  0.5 0.16   |    0.16 U 0.5     |      0.16       |    - - -    |      -      |    - -     |     -      |     -      |  ❌ *EMPTY*   | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  |      ❌ *EMPTY*      | ❌ *EMPTY*  | ❌ *EMPTY*  |
|                                    Cyclohexane                                     |                          0.0004                          |            0.27            |              U 10              |                       0.27                       |              0.27              |                 U 10                 |                                                       0.27                                                       |    0.27    |                         U 10                         |    0.27    |     0.27      |    U 10     |     0.27      |    0.27     |  ⚠️ **U**   |   10 0.27   |     0.27 U 10     |      0.27       |     - -     |      -      |     -      |     -      |    - -     |      -       | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  |      ❌ *EMPTY*      | ❌ *EMPTY*  | ❌ *EMPTY*  |
|                                    Ethylbenzene                                    |                            68                            |            0.17            |             U 0.5              |                       0.17                       |              0.17              |               ⚠️ **U**               |                                                       0.5                                                        |    0.17    |                         0.17                         |   U 0.5    |     0.17      |    0.17     |     U 0.5     |    0.17     | 🔴 **0.17 U** 🔴 |     0.5     |       0.17        |   0.17 U 0.5    |   0.17 -    |      -      |     -      |    - -     |     -      |      -       | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  |      ❌ *EMPTY*      | ❌ *EMPTY*  | ❌ *EMPTY*  |
|                                  Isopropylbenzene                                  |                            --                            |            0.19            |             U 0.5              |                       0.19                       |              0.19              |                U 0.5                 |                                                       0.19                                                       |    0.19    |                        U 0.5                         |    0.19    |     0.19      |    U 0.5    |   0.19 0.23   |    0.19     |    U 0.5    |    0.19     |    0.19 U 0.5     |     - 0.19      |    - - -    |      -      |    - -     |     -      |     -      |  ❌ *EMPTY*   | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  |      ❌ *EMPTY*      | ❌ *EMPTY*  | ❌ *EMPTY*  |
|                         Methyl Acetate Methyl cyclohexane                          |                            --                            |            0.23            |              U 2               |                       0.23                       |              0.23              |               ⚠️ **U**               |                                                        2                                                         |    0.23    |                         0.23                         |    U 2     |     0.23      |    0.23     |      U 2      |     0.4     | 🔴 **0.23 U** 🔴 |   2 0.23    | 0.23 U 2 0.4 U 10 |    0.23 - -     |      -      |      -      |     -      |    - -     |     -      |  ❌ *EMPTY*   | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  |      ❌ *EMPTY*      | ❌ *EMPTY*  | ❌ *EMPTY*  |
|                              Methyl tert butyl ether                               |                            --                            |            0.4             |              U 10              |                       0.4                        |              0.4               |               ⚠️ **U**               |                                                        10                                                        |    0.4     |                         0.4                          |    U 10    |      0.4      |     0.4     |     U 10      | 🔴 **0.4 U** 🔴 |     10      |  0.4 0.17   |        0.4        |      - - -      |      -      |     - -     |    - -     |     -      | ❌ *EMPTY*  |  ❌ *EMPTY*   | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  |      ❌ *EMPTY*      | ❌ *EMPTY*  | ❌ *EMPTY*  |
|                                      o-Xylene                                      |                            --                            |            0.17            |             U 1 1              |                    0.17 0.39                     |           0.17 0.39            |               ⚠️ **U**               |                                                        1                                                         | 0.17 0.39  |                      0.17 0.39                       |  U 1 U 1   |   0.17 0.39   |  0.17 0.39  |   ⚠️ **U**    |   1 0.17    |    0.17     |     U 1     |   0.17 U 1 U 1    |    0.17 0.39    |     - -     |     - -     |     -      |     -      |     -      |      -       | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  |      ❌ *EMPTY*      | ❌ *EMPTY*  | ❌ *EMPTY*  |
|                                     p/m-Xylene                                     |                          -- --                           |         0.39 0.33          |            ⚠️ **U**            |                       0.33                       |              0.33              |               ⚠️ **U**               |                                                        1                                                         |    0.33    |                         U 1                          |    0.33    |     0.33      |     U 1     |     0.39      | 🔴 **0.39 U** 🔴 |      1      | 0.39 0.39 U |       - - -       |       - -       |     - -     |      -      |    - -     |    - -     |    - -     |  ❌ *EMPTY*   | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  |      ❌ *EMPTY*      | ❌ *EMPTY*  | ❌ *EMPTY*  |
|                                      Styrene                                       |                            --                            |            0.36            |            U 1 U 1             |                       0.36                       |              U 1               |                 0.33                 |                                                       0.36                                                       |    U 1     |                         0.33                         | 🔴 **0.33 U** 🔴 |       1       | 0.33 0.33 U | 1 0.33 1 0.36 |      -      |     - -     |      -      |        - -        |        -        |  ❌ *EMPTY*  |  ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  |  ❌ *EMPTY*   | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  |      ❌ *EMPTY*      | ❌ *EMPTY*  | ❌ *EMPTY*  |
|                                      Toluene                                       |                            57                            |            0.2             |             U 0.75             |                       0.36                       |              U 1               |                 0.36                 |                                                       0.36                                                       |    U 1     |                       0.36 0.2                       | U 1 U 0.75 | 0.36 0.36 0.2 |   U 0.2 U   |       1       |    0.36     |    0.36     |     0.2     |         -         |       - -       |    - - -    |      -      |    - -     | ❌ *EMPTY*  | ❌ *EMPTY*  |  ❌ *EMPTY*   | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  |      ❌ *EMPTY*      | ❌ *EMPTY*  | ❌ *EMPTY*  |
|                                        0.2                                         |                           0.2                            |          ⚠️ **U**          |              0.75              |                       0.2                        |              0.2               |                U 0.75                |                                                       0.2                                                        |    0.75    |                         0.2                          | 0.2 U 0.75 |      - -      |  ❌ *EMPTY*  |   ❌ *EMPTY*   |  ❌ *EMPTY*  |  ❌ *EMPTY*  |  ❌ *EMPTY*  |     ❌ *EMPTY*     |    ❌ *EMPTY*    |  ❌ *EMPTY*  |  ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  |  ❌ *EMPTY*   | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  |      ❌ *EMPTY*      | ❌ *EMPTY*  | ❌ *EMPTY*  |
|                                   Xylenes, Total                                   |                           210                            |            0.33            |              U 1               |                       0.33                       |              0.33              |               ⚠️ **U**               |                                                        1                                                         |    0.33    |                         0.33                         |    U 1     |     0.33      |    0.33     |      U 1      |  0.33 0.33  |  ⚠️ **U**   |      1      |       0.33        |    0.33 U 1     |      -      |    0.33     |     -      |     -      |    - -     |  ❌ *EMPTY*   | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  |      ❌ *EMPTY*      | ❌ *EMPTY*  | ❌ *EMPTY*  |
|                         - - Semivolatile Organics by GC/MS                         |                        ❌ *EMPTY*                         |         ❌ *EMPTY*          |           ❌ *EMPTY*            |                    ❌ *EMPTY*                     |           ❌ *EMPTY*            |              ❌ *EMPTY*               |                                                    ❌ *EMPTY*                                                     | ❌ *EMPTY*  |                      ❌ *EMPTY*                       | ❌ *EMPTY*  |   ❌ *EMPTY*   |  ❌ *EMPTY*  |   ❌ *EMPTY*   |  ❌ *EMPTY*  |  ❌ *EMPTY*  |  ❌ *EMPTY*  |     ❌ *EMPTY*     |    ❌ *EMPTY*    |  ❌ *EMPTY*  |  ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  |  ❌ *EMPTY*   | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  |      ❌ *EMPTY*      | ❌ *EMPTY*  | ❌ *EMPTY*  |
|                             1,2,4,5-Tetrachlorobenzene                             |                           0.03                           |            0.44            |             U 1.7              |                       0.44                       |               -                |                -   -                 |                                                        -                                                         |     -      |                        -   -                         |     -      |       -       |    -   -    |       -       |      -      |      -      |    -   -    |     -   -   -     |        -        |      -      |     - -     |    - -     |     -      |    - -     |  ❌ *EMPTY*   | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  |      ❌ *EMPTY*      | ❌ *EMPTY*  | ❌ *EMPTY*  |
|                                2-Methylnaphthalene                                 |                            --                            |            0.45            |              U 2               |                       0.45                       |               -                |                  -                   |                                                        -                                                         |     -      |                          -                           |    - -     |       -       |      -      |      - -      |      -      |     - -     |     - -     |        - -        |     - - - -     |      -      |     - -     |     -      |     -      |     -      |  ❌ *EMPTY*   | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  |      ❌ *EMPTY*      | ❌ *EMPTY*  | ❌ *EMPTY*  |
|                                    Acenaphthene                                    |                            17                            |            0.53            |              U 2               |                       0.53                       |               -                |                  -                   |                                                        -                                                         |     -      |                         - -                          |     -      |       -       |      -      |      - -      |      -      |     - -     |     - -     |        - -        |       - -       |    - - -    |     - -     |     -      |    - -     |    - -     |      -       | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  |      ❌ *EMPTY*      | ❌ *EMPTY*  | ❌ *EMPTY*  |
|                                     Anthracene                                     |                           300                            |            0.33            |              U 2               |                       0.33                       |               -                |                 - -                  |                                                        -                                                         |     -      |                         - -                          |     -      |       -       |     - -     |      - -      |     - -     |     - -     |    - - -    |        - -        |       - -       |    - - -    |     - -     |     -      | ❌ *EMPTY*  | ❌ *EMPTY*  |  ❌ *EMPTY*   | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  |      ❌ *EMPTY*      | ❌ *EMPTY*  | ❌ *EMPTY*  |
|                                      Biphenyl                                      |                            --                            |            0.46            |              U 2               |                       0.46                       |               -                |                 - -                  |                                                       - -                                                        |    - -     |                       - - - -                        |     -      |       -       |     - -     |      - -      |      -      |      -      |      -      |        - -        |       - -       |     - -     |     - -     |    - -     |   - - -    |   - - -    |      -       |    - -     | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  |      ❌ *EMPTY*      | ❌ *EMPTY*  | ❌ *EMPTY*  |
|                                      Chrysene                                      |                           0.12                           |          0.34 0.5          |           U 1.4 U 2            |                     0.34 0.5                     |              - -               |                 - -                  |                                                        -                                                         |     -      |                        - - -                         |    - -     |       -       |      -      |      - -      |    - - -    |   - - - -   | - - - - - - |       - - -       |        -        |    - - -    |     - -     |     -      |     -      |     -      |  ❌ *EMPTY*   | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  |      ❌ *EMPTY*      | ❌ *EMPTY*  | ❌ *EMPTY*  |
|                              Dibenzofuran Fluorene -                               |                            --                            |            0.26            |             U 2 2              |                       0.26                       |               -                |                - - -                 |                                                       - -                                                        |    - -     |                        - - -                         |     -      |       -       |     - -     |       -       |    - - -    |    - - -    |   - - - -   |      - - - -      |   - - - - - -   |     - -     |     - -     |   - - -    |     -      |     -      | Fluoranthene |    - -     |     -      |     -      |     -      |     20     |     -      |  ⚠️ **U**  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  | ❌ *EMPTY*  |      ❌ *EMPTY*      | ❌ *EMPTY*  | ❌ *EMPTY*  |
|                                    Naphthalene                                     |                          50 43                           |       0.41 0.33 0.28       |           0.46 2 2 2           |                    0.41 0.46                     |              - -               |                - - -                 |                                                      - - -                                                       |   - - -    |                        - - -                         |  - - - -   |   - - - - -   |      -      |      - -      |      -      | - - - - - - |    - - -    |    - - - - - -    |  - - - - - - -  | - - - - - - | - - - - - - |   - - -    |     -      |     -      |     - -      |  - - - -   |     -      |   - - -    |     -      |   - - -    |     -      |    1 20    |    0.33    |    - -     |   - - -    |    - -     | Phenanthrene Pyrene |   U U U    |    0.28    |

### 🔍 Pattern Analysis for this Table:

**❌ 6 misplaced qualifiers found!**
- 🔴 **Misplaced qualifier detected:** `1.7 J` (should be split)
- 🔴 **Misplaced qualifier detected:** `0.17 U` (should be split)
- 🔴 **Misplaced qualifier detected:** `0.23 U` (should be split)
- 🔴 **Misplaced qualifier detected:** `0.4 U` (should be split)
- 🔴 **Misplaced qualifier detected:** `0.39 U` (should be split)
- ... and 1 more
⚠️ **19 standalone qualifiers found** (verify they're in correct columns)

---


## 📋 Table 4 - Environmental Lab Data

> **⚠️ Document Convention Analysis:**
> - **U** = Undetected (below detection limit) - should be in separate qualifier column
> - **J** = Estimated value (detected but below reporting limit) - should be in separate qualifier column
> - **Expected Pattern:** Concentration | Qualifier | Reporting Limit | Method Detection Limit
> - **Look for:** Values like '0.046 U' that should be split into '0.046' and 'U'

|    Soil Sampling Locations with Exceedances     |        Soil Sampling Locations with Exceedances         |                     Soil Sampling Locations with Exceedances                      |                       Soil Sampling Locations with Exceedances                        |
|:-----------------------------------------------:|:-------------------------------------------------------:|:---------------------------------------------------------------------------------:|:-------------------------------------------------------------------------------------:|
|                    Parameter                    |                          Area                           | Pennsylvania Non-Residential  Direct Contact Surface Soil (0- 2') Screening Value | Pennsylvania Non-Residential  Direct Contact Subsurface Soil  (2-15') Screening Value | Applicable Pennsylvania Non- Residential Soil to Groundwater  Screening Value (Higher of  Generic and 100 X GW MSC) | Pennsylvania Non-Residential  Soil SHS Vapor Intrusion  Screening Value |
|                    AST Case                     |                        ❌ *EMPTY*                        |                                     ❌ *EMPTY*                                     |                                       ❌ *EMPTY*                                       |                                                      ❌ *EMPTY*                                                      |                                ❌ *EMPTY*                                |
|                     Benzene                     |                        280 mg/kg                        |                                     330 mg/kg                                     |                                       0.5 mg/kg                                       |                                                     0.13 mg/kg                                                      |                                ❌ *EMPTY*                                |
|             Tank Containment Area 1             |              7551-P6 (3 ft)  [0.72 mg/kg]               |                           7551-P6 (3 ft)  [0.72 mg/kg]                            |                                       ❌ *EMPTY*                                       |                                                      ❌ *EMPTY*                                                      |                                ❌ *EMPTY*                                |
|             Tank Containment Area 1             |              UNK-ST-N (3 ft)  [0.56 mg/kg]              |                           UNK-ST-N (3 ft)  [0.56 mg/kg]                           |                                       ❌ *EMPTY*                                       |                                                      ❌ *EMPTY*                                                      |                                ❌ *EMPTY*                                |
|             Tank Containment Area 3             |               Pipe 29(2 ft)  [0.66 mg/kg]               |                            Pipe 29(2 ft)  [0.66 mg/kg]                            |                                       ❌ *EMPTY*                                       |                                                      ❌ *EMPTY*                                                      |                                ❌ *EMPTY*                                |
|             Tank Containment Area 1             |             SB-105 (4-4.5 ft)  [1.8 mg/kg]              |                          SB-105 (4-4.5 ft)  [1.8 mg/kg]                           |                                       ❌ *EMPTY*                                       |                                                      ❌ *EMPTY*                                                      |                                ❌ *EMPTY*                                |
|             Tank Containment Area 1             |             SB-213 (4-4.5 ft)  [1.8 mg/kg ]             |                          SB-213 (4-4.5 ft)  [1.8 mg/kg ]                          |                                       ❌ *EMPTY*                                       |                                                      ❌ *EMPTY*                                                      |                                ❌ *EMPTY*                                |
|             Tank Containment Area 1             |             SB-207 (4-4.5 ft)  [0.19 mg/kg]             |                                     ❌ *EMPTY*                                     |                                       ❌ *EMPTY*                                       |                                                      ❌ *EMPTY*                                                      |                                ❌ *EMPTY*                                |
|             Tank Containment Area 1             |             SB-216 (3.5-4 ft)  [0.27 mg/kg]             |                                     ❌ *EMPTY*                                     |                                       ❌ *EMPTY*                                       |                                                      ❌ *EMPTY*                                                      |                                ❌ *EMPTY*                                |
|                   Naphthalene                   |                        66 mg/kg                         |                                     77 mg/kg                                      |                                       25 mg/kg                                        |                                                      25 mg/kg                                                       |                                ❌ *EMPTY*                                |
|             Tank Containment Area 1             |            SB-213 (4-4.5 ft)  [1,700 mg/kg]             |                           SB-215 (4.5-5 ft)  [38 mg/kg]                           |                           SB-213 (4-4.5 ft)  [1,700 mg/kg]                            |                                                      ❌ *EMPTY*                                                      |                                ❌ *EMPTY*                                |
|             Tank Containment Area 1             |              SB-215 (4.5-5 ft)  [38 mg/kg]              |                                     ❌ *EMPTY*                                     |                                       ❌ *EMPTY*                                       |                                                      ❌ *EMPTY*                                                      |                                ❌ *EMPTY*                                |
|                   Lead, total                   |                       1,000 mg/kg                       |                                   190,000 mg/kg                                   |                                       450 mg/kg                                       |                                                         NA                                                          |                                ❌ *EMPTY*                                |
|             Tank Containment Area 1             |               7550-P1 (3 ft)  [690 mg/kg]               |                                     ❌ *EMPTY*                                     |                                       ❌ *EMPTY*                                       |                                                      ❌ *EMPTY*                                                      |                                ❌ *EMPTY*                                |
|             Tank Containment Area 1             |               Pipe 70 (2 ft)  [750 mg/kg]               |                                     ❌ *EMPTY*                                     |                                       ❌ *EMPTY*                                       |                                                      ❌ *EMPTY*                                                      |                                ❌ *EMPTY*                                |
|             Tank Containment Area 1             |               Pipe 81 (2 ft)  [500 mg/kg]               |                                     ❌ *EMPTY*                                     |                                       ❌ *EMPTY*                                       |                                                      ❌ *EMPTY*                                                      |                                ❌ *EMPTY*                                |
|             Tank Containment Area 1             |               Pipe 86 (2 ft)  [540 mg/kg]               |                                     ❌ *EMPTY*                                     |                                       ❌ *EMPTY*                                       |                                                      ❌ *EMPTY*                                                      |                                ❌ *EMPTY*                                |
|             Tank Containment Area 1             |               Pipe 87 (2 ft)  [720 mg/kg]               |                                     ❌ *EMPTY*                                     |                                       ❌ *EMPTY*                                       |                                                      ❌ *EMPTY*                                                      |                                ❌ *EMPTY*                                |
|                   Act 2 Case                    |                        ❌ *EMPTY*                        |                                     ❌ *EMPTY*                                     |                                       ❌ *EMPTY*                                       |                                                      ❌ *EMPTY*                                                      |                                ❌ *EMPTY*                                |
|                     Benzene                     |                        280 mg/kg                        |                                     330 mg/kg                                     |                                       0.5 mg/kg                                       |                                                     0.13 mg/kg                                                      |                                ❌ *EMPTY*                                |
|       Pad West of Tank Containment Area 1       |               SB-202 (8.5-9)  [2.6 mg/kg]               |                            SB-202 (8.5-9)  [2.6 mg/kg]                            |                                       ❌ *EMPTY*                                       |                                                      ❌ *EMPTY*                                                      |                                ❌ *EMPTY*                                |
|       Pad West of Tank Containment Area 1       |              SB-204 (8.5-9)  [0.59 mg/kg]               |                           SB-204 (8.5-9)  [0.59 mg/kg]                            |                                       ❌ *EMPTY*                                       |                                                      ❌ *EMPTY*                                                      |                                ❌ *EMPTY*                                |
|       Pad West of Tank Containment Area 1       |                SB-217 (8.5-9)  [1 mg/kg]                |                             SB-217 (8.5-9)  [1 mg/kg]                             |                                       ❌ *EMPTY*                                       |                                                      ❌ *EMPTY*                                                      |                                ❌ *EMPTY*                                |
|       Pad West of Tank Containment Area 1       |               SB-203 (8.5-9)  [0.4 mg/kg]               |                                     ❌ *EMPTY*                                     |                                       ❌ *EMPTY*                                       |                                                      ❌ *EMPTY*                                                      |                                ❌ *EMPTY*                                |
|                   Lead, total                   |                       1,000 mg/kg                       |                                   190,000 mg/kg                                   |                                       450 mg/kg                                       |                                                         NA                                                          |                                ❌ *EMPTY*                                |
|             Tank Containment Area 3             |               941-P3 (3 ft)  [610 mg/kg]                |                                     ❌ *EMPTY*                                     |                                       ❌ *EMPTY*                                       |                                                      ❌ *EMPTY*                                                      |                                ❌ *EMPTY*                                |
|             Tank Containment Area 3             |             941-Center (5 ft)  [560 mg/kg]              |                                     ❌ *EMPTY*                                     |                                       ❌ *EMPTY*                                       |                                                      ❌ *EMPTY*                                                      |                                ❌ *EMPTY*                                |
|             Tank Containment Area 2             |               1043-P2 (3 ft)  [490 mg/kg]               |                                     ❌ *EMPTY*                                     |                                       ❌ *EMPTY*                                       |                                                      ❌ *EMPTY*                                                      |                                ❌ *EMPTY*                                |
|             Tank Containment Area 2             |               1043-P4 (3 ft)  [880 mg/kg]               |                                     ❌ *EMPTY*                                     |                                       ❌ *EMPTY*                                       |                                                      ❌ *EMPTY*                                                      |                                ❌ *EMPTY*                                |
|             Tank Containment Area 2             |               1043-P5 (3 ft)  [560 mg/kg]               |                                     ❌ *EMPTY*                                     |                                       ❌ *EMPTY*                                       |                                                      ❌ *EMPTY*                                                      |                                ❌ *EMPTY*                                |
|             Tank Containment Area 2             |               1044-P3 (3 ft)  [810 mg/kg]               |                                     ❌ *EMPTY*                                     |                                       ❌ *EMPTY*                                       |                                                      ❌ *EMPTY*                                                      |                                ❌ *EMPTY*                                |
|             Tank Containment Area 2             |              1044-P4 (3 ft)  [2,100 mg/kg]              |                                     ❌ *EMPTY*                                     |                                       ❌ *EMPTY*                                       |                                                      ❌ *EMPTY*                                                      |                                ❌ *EMPTY*                                |
|             Tank Containment Area 2             |             1044-Center (5 ft)  [990 mg/kg]             |                                     ❌ *EMPTY*                                     |                                       ❌ *EMPTY*                                       |                                                      ❌ *EMPTY*                                                      |                                ❌ *EMPTY*                                |
|             Tank Containment Area 3             |              2040-P2 (3 ft)  [1,200 mg/kg]              |                                     ❌ *EMPTY*                                     |                                       ❌ *EMPTY*                                       |                                                      ❌ *EMPTY*                                                      |                                ❌ *EMPTY*                                |
|             Tank Containment Area 3             |                        ❌ *EMPTY*                        |                                     ❌ *EMPTY*                                     |                                       ❌ *EMPTY*                                       |                                                      ❌ *EMPTY*                                                      |                                ❌ *EMPTY*                                |
|           Pipe 33 (2 ft)  [920 mg/kg]           |                        ❌ *EMPTY*                        |                                     ❌ *EMPTY*                                     |                                       ❌ *EMPTY*                                       |                                                      ❌ *EMPTY*                                                      |                                ❌ *EMPTY*                                |
| Tank Containment Area 2 Tank Containment Area 2 | Pipe 46 (2 ft)  [600 mg/kg] Pipe 51 (2 ft)  [460 mg/kg] |                                     ❌ *EMPTY*                                     |                                       ❌ *EMPTY*                                       |                                                      ❌ *EMPTY*                                                      |                                ❌ *EMPTY*                                |
|             Tank Containment Area 2             |               Pipe 55 (2 ft)  [750 mg/kg]               |                                     ❌ *EMPTY*                                     |                                       ❌ *EMPTY*                                       |                                                      ❌ *EMPTY*                                                      |                                ❌ *EMPTY*                                |

### 🔍 Pattern Analysis for this Table:

✅ **No obviously misplaced qualifiers detected**

---


## 📋 Table 5 - Environmental Lab Data

> **⚠️ Document Convention Analysis:**
> - **U** = Undetected (below detection limit) - should be in separate qualifier column
> - **J** = Estimated value (detected but below reporting limit) - should be in separate qualifier column
> - **Expected Pattern:** Concentration | Qualifier | Reporting Limit | Method Detection Limit
> - **Look for:** Values like '0.046 U' that should be split into '0.046' and 'U'

|    ELEVATIONS    |  ELEVATIONS   | COORDINATES | COORDINATES  |  COORDINATES   |  COORDINATES  |
|:----------------:|:-------------:|:-----------:|:------------:|:--------------:|:-------------:|
| MONITORING WELLS |    GROUND     |     RIM     | PVC NORTHING |    EASTING     | LATITUDE (N)  | LONGITUDE (W) |
|       MW-1       |  12.51' DIRT  |   15.07'    | 14.19 229614 |    2681439     | 39°12'24.78"  | 66°46'41.29"  |
|       MW-2       |  15.11' DIRT  |   17.52'    | 16.44 229692 |    2681017     | 39°12'25.90"  | 66°46'46.52"  |
|       MW-3       |  16.70' DIRT  |   19.99'    |    19.33     | 229431 2680977 | 39°12'23.38"  | 66°46'47.31"  |
|       MW-4       | 27.51' GRAVEL |   30.44'    | 28.76 229309 |    2680696     | 39°12'22.42"  | 66°46'50.97"  |
|       MW-5       | 26.70' GRAVEL |   29.22'    | 29.02 229565 |    2680481     | 39°12'25.11"  | 66°46'53.40"  |
|       MW-7       |  16.43' DIRT  |   19.47'    | 19.17 229682 |    2680734     | 39°12'26.05"  | 66°46'50.09"  |
|       MW-8       |  23.18' PAVE  |   23.09'    | 22.74 229715 |    2680555     | 39°12'26.52"  | 66°46'52.30"  |
|       MW-9       | 29.03' GRAVEL |   31.54'    | 31.36 229269 |    2680544     | 39°12'22.16"  | 66°46'52.93"  |

### 🔍 Pattern Analysis for this Table:

✅ **No obviously misplaced qualifiers detected**

---


## 📋 Table 6 - Environmental Lab Data

> **⚠️ Document Convention Analysis:**
> - **U** = Undetected (below detection limit) - should be in separate qualifier column
> - **J** = Estimated value (detected but below reporting limit) - should be in separate qualifier column
> - **Expected Pattern:** Concentration | Qualifier | Reporting Limit | Method Detection Limit
> - **Look for:** Values like '0.046 U' that should be split into '0.046' and 'U'

|        Analyte         | Result Qualifier |     RL     |    MDL     |    Unit    | Dil Fac D Method | Prep Type  |
|:----------------------:|:----------------:|:----------:|:----------:|:----------:|:----------------:|:----------:|
| 1,3,5-Trimethylbenzene |       8100       |    360     |     36     |   ug/Kg    |        50        |     ☼      |   8260C    |  Total/NA  | ❌ *EMPTY*  |
|        Toluene         |       100        |  ⚠️ **J**  |    360     |     44     |      ug/Kg       |     50     |     ☼      |   8260C    |  Total/NA  |
|     Xylenes, Total     |       3500       |    730     |    100     |   ug/Kg    |        50        |     ☼      |   8260C    |  Total/NA  | ❌ *EMPTY*  |
|      Naphthalene       |       2000       |    360     |    150     |   ug/Kg    |        50        |     ☼      |   8260C    |  Total/NA  | ❌ *EMPTY*  |
| 1,2,4-Trimethylbenzene |      19000       |    360     |     36     |   ug/Kg    |        50        |     ☼      |   8260C    |  Total/NA  | ❌ *EMPTY*  |
|    Isopropylbenzene    |       1800       |    360     |     29     |   ug/Kg    |        50        |     ☼      |   8260C    |  Total/NA  | ❌ *EMPTY*  |
|   Benzo[a]anthracene   |       190        |     22     |    4.5     |   ug/Kg    |        1         |     ☼      |   8270D    |  Total/NA  | ❌ *EMPTY*  |
|     Benzo[a]pyrene     |       270        |     22     |    4.5     |   ug/Kg    |        1         |     ☼      |   8270D    |  Total/NA  | ❌ *EMPTY*  |
|  Benzo[b]fluoranthene  |       200        |     22     |    4.5     |   ug/Kg    |        1         |     ☼      |   8270D    |  Total/NA  | ❌ *EMPTY*  |
|  Benzo[g,h,i]perylene  |       270        |     22     |    4.5     |   ug/Kg    |        1         |     ☼      |   8270D    |  Total/NA  | ❌ *EMPTY*  |
|        Chrysene        |       370        |     22     |    4.5     |   ug/Kg    |        1         |     ☼      |   8270D    |  Total/NA  | ❌ *EMPTY*  |
|        Fluorene        |       900        |     22     |    4.5     |   ug/Kg    |        1         |     ☼      |   8270D    |  Total/NA  | ❌ *EMPTY*  |
|      Phenanthrene      |       870        |     22     |    5.4     |   ug/Kg    |        1         |     ☼      |   8270D    |  Total/NA  | ❌ *EMPTY*  |
|         Pyrene         |       560        |     22     |    4.5     |   ug/Kg    |        1         |     ☼      |   8270D    |  Total/NA  | ❌ *EMPTY*  |
|          Lead          |       210        |    1.8     |    0.71    |   mg/Kg    |        1         |     ☼      |   6010C    |  Total/NA  | ❌ *EMPTY*  |

### 🔍 Pattern Analysis for this Table:

✅ **No obviously misplaced qualifiers detected**
⚠️ **1 standalone qualifiers found** (verify they're in correct columns)

---


## 📋 Table 7 - Environmental Lab Data

> **⚠️ Document Convention Analysis:**
> - **U** = Undetected (below detection limit) - should be in separate qualifier column
> - **J** = Estimated value (detected but below reporting limit) - should be in separate qualifier column
> - **Expected Pattern:** Concentration | Qualifier | Reporting Limit | Method Detection Limit
> - **Look for:** Values like '0.046 U' that should be split into '0.046' and 'U'

|           Analyte           | Result Qualifier |     RL     |    MDL     |    Unit    | Dil Fac D Method | Prep Type  |
|:---------------------------:|:----------------:|:----------:|:----------:|:----------:|:----------------:|:----------:|
| Methyl tertiary butyl ether |       1.1        |  ⚠️ **J**  |    4.8     |    0.48    |      ug/Kg       |     1      |  ☼ 8260C   |  Total/NA  |
|         Anthracene          |        14        |  ⚠️ **J**  |     18     |    3.6     |      ug/Kg       |     1      |  ☼ 8270D   |  Total/NA  |
|     Benzo[a]anthracene      |        83        |     18     |    3.6     |   ug/Kg    |        1         |  ☼ 8270D   |  Total/NA  | ❌ *EMPTY*  |
|       Benzo[a]pyrene        |       120        |     18     |    3.6     |   ug/Kg    |        1         |  ☼ 8270D   |  Total/NA  | ❌ *EMPTY*  |
|    Benzo[b]fluoranthene     |       190        |     18     |    3.6     |   ug/Kg    |        1         |  ☼ 8270D   |  Total/NA  | ❌ *EMPTY*  |
|    Benzo[g,h,i]perylene     |       150        |     18     |    3.6     |   ug/Kg    |        1         |  ☼ 8270D   |  Total/NA  | ❌ *EMPTY*  |
|          Chrysene           |       140        |     18     |    3.6     |   ug/Kg    |        1         |  ☼ 8270D   |  Total/NA  | ❌ *EMPTY*  |
|          Fluorene           |       5.6        |  ⚠️ **J**  |     18     |    3.6     |      ug/Kg       |     1      |  ☼ 8270D   |  Total/NA  |
|        Phenanthrene         |        76        |     18     |    4.3     |   ug/Kg    |        1         |  ☼ 8270D   |  Total/NA  | ❌ *EMPTY*  |
|           Pyrene            |       180        |     18     |    3.6     |   ug/Kg    |        1         |  ☼ 8270D   |  Total/NA  | ❌ *EMPTY*  |
|            Lead             |       270        |    1.6     |    0.62    |   mg/Kg    |        1         |  ☼ 6010C   |  Total/NA  | ❌ *EMPTY*  |

### 🔍 Pattern Analysis for this Table:

✅ **No obviously misplaced qualifiers detected**
⚠️ **3 standalone qualifiers found** (verify they're in correct columns)

---


## 📋 Table 8 - Environmental Lab Data

> **⚠️ Document Convention Analysis:**
> - **U** = Undetected (below detection limit) - should be in separate qualifier column
> - **J** = Estimated value (detected but below reporting limit) - should be in separate qualifier column
> - **Expected Pattern:** Concentration | Qualifier | Reporting Limit | Method Detection Limit
> - **Look for:** Values like '0.046 U' that should be split into '0.046' and 'U'

|        Analyte         | Result Qualifier |     RL     |    MDL     |    Unit    | Dil Fac D Method | Prep Type  |
|:----------------------:|:----------------:|:----------:|:----------:|:----------:|:----------------:|:----------:|
| 1,3,5-Trimethylbenzene |       0.71       |  ⚠️ **J**  |    6.3     |    0.63    |      ug/Kg       |     1      |  ☼ 8260C   |  Total/NA  |
|        Toluene         |       0.84       |  ⚠️ **J**  |    6.3     |    0.75    |      ug/Kg       |     1      |  ☼ 8260C   |  Total/NA  |
|        Benzene         |       0.85       |  ⚠️ **J**  |    6.3     |    0.63    |      ug/Kg       |     1      |  ☼ 8260C   |  Total/NA  |
| 1,2,4-Trimethylbenzene |       0.77       |  ⚠️ **J**  |    6.3     |    0.63    |      ug/Kg       |     1      |  ☼ 8260C   |  Total/NA  |
|       Anthracene       |        65        |     23     |    4.6     |   ug/Kg    |        1         |  ☼ 8270D   |  Total/NA  | ❌ *EMPTY*  |
|   Benzo[a]anthracene   |       220        |     23     |    4.6     |   ug/Kg    |        1         |  ☼ 8270D   |  Total/NA  | ❌ *EMPTY*  |
|     Benzo[a]pyrene     |       200        |     23     |    4.6     |   ug/Kg    |        1         |  ☼ 8270D   |  Total/NA  | ❌ *EMPTY*  |
|  Benzo[b]fluoranthene  |       230        |     23     |    4.6     |   ug/Kg    |        1         |  ☼ 8270D   |  Total/NA  | ❌ *EMPTY*  |
|  Benzo[g,h,i]perylene  |       140        |     23     |    4.6     |   ug/Kg    |        1         |  ☼ 8270D   |  Total/NA  | ❌ *EMPTY*  |
|        Chrysene        |       210        |     23     |    4.6     |   ug/Kg    |        1         |  ☼ 8270D   |  Total/NA  | ❌ *EMPTY*  |
|        Fluorene        |        32        |     23     |    4.6     |   ug/Kg    |        1         |  ☼ 8270D   |  Total/NA  | ❌ *EMPTY*  |
|      Phenanthrene      |       260        |     23     |    5.5     |   ug/Kg    |        1         |  ☼ 8270D   |  Total/NA  | ❌ *EMPTY*  |
|         Pyrene         |       340        |     23     |    4.6     |   ug/Kg    |        1         |  ☼ 8270D   |  Total/NA  | ❌ *EMPTY*  |
|          Lead          |        44        |    1.9     |    0.76    |   mg/Kg    |        1         |  ☼ 6010C   |  Total/NA  | ❌ *EMPTY*  |

### 🔍 Pattern Analysis for this Table:

✅ **No obviously misplaced qualifiers detected**
⚠️ **4 standalone qualifiers found** (verify they're in correct columns)

---


## 📋 Table 9 - Environmental Lab Data

> **⚠️ Document Convention Analysis:**
> - **U** = Undetected (below detection limit) - should be in separate qualifier column
> - **J** = Estimated value (detected but below reporting limit) - should be in separate qualifier column
> - **Expected Pattern:** Concentration | Qualifier | Reporting Limit | Method Detection Limit
> - **Look for:** Values like '0.046 U' that should be split into '0.046' and 'U'

|       Analyte        | Result Qualifier |     RL     |  MDL Unit  | Dil Fac D Method | Prep Type  |
|:--------------------:|:----------------:|:----------:|:----------:|:----------------:|:----------:|
|  Benzo[a]anthracene  |      J 170       |    220     |  ug/Kg 45  |       ☼ 5        |   8270D    |  Total/NA  |
|    Benzo[a]pyrene    |       220        |    220     |  ug/Kg 45  |       ☼ 5        |   8270D    |  Total/NA  |
| Benzo[b]fluoranthene |       230        |    220     |  ug/Kg 45  |       ☼ 5        |   8270D    |  Total/NA  |
| Benzo[g,h,i]perylene |  🔴 **160 J** 🔴   |    220     |  ug/Kg 45  |       ☼ 5        |   8270D    |  Total/NA  |
|       Chrysene       |  🔴 **170 J** 🔴   |    220     |  ug/Kg 45  |       ☼ 5        |   8270D    |  Total/NA  |
|     Phenanthrene     |   🔴 **56 J** 🔴   |    220     |  ug/Kg 54  |       ☼ 5        |   8270D    |  Total/NA  |
|        Pyrene        |       220        |    220     |  ug/Kg 45  |       ☼ 5        |   8270D    |  Total/NA  |
|         Lead         |        83        |    1.4     | mg/Kg 0.58 |       ☼ 1        |   6010C    |  Total/NA  |

### 🔍 Pattern Analysis for this Table:

**❌ 3 misplaced qualifiers found!**
- 🔴 **Misplaced qualifier detected:** `160 J` (should be split)
- 🔴 **Misplaced qualifier detected:** `170 J` (should be split)
- 🔴 **Misplaced qualifier detected:** `56 J` (should be split)

---


## 📋 Table 10 - Environmental Lab Data

> **⚠️ Document Convention Analysis:**
> - **U** = Undetected (below detection limit) - should be in separate qualifier column
> - **J** = Estimated value (detected but below reporting limit) - should be in separate qualifier column
> - **Expected Pattern:** Concentration | Qualifier | Reporting Limit | Method Detection Limit
> - **Look for:** Values like '0.046 U' that should be split into '0.046' and 'U'

|       Analyte        | Result Qualifier |     RL     |    MDL     |    Unit    | Dil Fac D Method | Prep Type  |
|:--------------------:|:----------------:|:----------:|:----------:|:----------:|:----------------:|:----------:|
|       Benzene        |       1.3        |  ⚠️ **J**  |    6.7     |    0.67    |      ug/Kg       |    ☼ 1     |   8260C    |  Total/NA  |
|      Anthracene      |        64        |     23     |    4.5     |   ug/Kg    |       ☼ 1        |   8270D    |  Total/NA  | ❌ *EMPTY*  |
|  Benzo[a]anthracene  |       120        |     23     |    4.5     |   ug/Kg    |       ☼ 1        |   8270D    |  Total/NA  | ❌ *EMPTY*  |
|    Benzo[a]pyrene    |        95        |     23     |    4.5     |   ug/Kg    |       ☼ 1        |   8270D    |  Total/NA  | ❌ *EMPTY*  |
| Benzo[b]fluoranthene |       120        |     23     |    4.5     |   ug/Kg    |       ☼ 1        |   8270D    |  Total/NA  | ❌ *EMPTY*  |
| Benzo[g,h,i]perylene |        77        |     23     |    4.5     |   ug/Kg    |       ☼ 1        |   8270D    |  Total/NA  | ❌ *EMPTY*  |
|       Chrysene       |       120        |     23     |    4.5     |   ug/Kg    |       ☼ 1        |   8270D    |  Total/NA  | ❌ *EMPTY*  |
|       Fluorene       |        37        |     23     |    4.5     |   ug/Kg    |       ☼ 1        |   8270D    |  Total/NA  | ❌ *EMPTY*  |
|     Phenanthrene     |        99        |     23     |    5.4     |   ug/Kg    |        1         |  ☼ 8270D   |  Total/NA  | ❌ *EMPTY*  |
|        Pyrene        |       150        |     23     |    4.5     |   ug/Kg    |        1         |  ☼ 8270D   |  Total/NA  | ❌ *EMPTY*  |
|         Lead         |       2100       |    1.9     |    0.78    |   mg/Kg    |       ☼ 1        |   6010C    |  Total/NA  | ❌ *EMPTY*  |

### 🔍 Pattern Analysis for this Table:

✅ **No obviously misplaced qualifiers detected**
⚠️ **1 standalone qualifiers found** (verify they're in correct columns)

---


## 📋 Table 11 - Environmental Lab Data

> **⚠️ Document Convention Analysis:**
> - **U** = Undetected (below detection limit) - should be in separate qualifier column
> - **J** = Estimated value (detected but below reporting limit) - should be in separate qualifier column
> - **Expected Pattern:** Concentration | Qualifier | Reporting Limit | Method Detection Limit
> - **Look for:** Values like '0.046 U' that should be split into '0.046' and 'U'

|       Analyte        | Result Qualifier |     RL     |    MDL     |    Unit    | Dil Fac D Method | Prep Type  |
|:--------------------:|:----------------:|:----------:|:----------:|:----------:|:----------------:|:----------:|
|      Anthracene      |       110        |     23     |    4.7     |   ug/Kg    |       ☼ 1        |   8270D    |  Total/NA  |
|  Benzo[a]anthracene  |       510        |     23     |    4.7     |   ug/Kg    |       ☼ 1        |   8270D    |  Total/NA  |
|    Benzo[a]pyrene    |       460        |     23     |    4.7     |   ug/Kg    |        1         |  ☼ 8270D   |  Total/NA  |
| Benzo[b]fluoranthene |       580        |     23     |    4.7     |   ug/Kg    |        1         |  ☼ 8270D   |  Total/NA  |
| Benzo[g,h,i]perylene |       330        |     23     |    4.7     |   ug/Kg    |        1         |  ☼ 8270D   |  Total/NA  |
|       Chrysene       |       580        |     23     |    4.7     |   ug/Kg    |        1         |  ☼ 8270D   |  Total/NA  |
|       Fluorene       |        92        |     23     |    4.7     |   ug/Kg    |        1         |  ☼ 8270D   |  Total/NA  |
|     Phenanthrene     |       1100       |     23     |    5.6     |   ug/Kg    |        1         |  ☼ 8270D   |  Total/NA  |
|        Pyrene        |       930        |     23     |    4.7     |   ug/Kg    |        1         |  ☼ 8270D   |  Total/NA  |
|         Lead         |      750 F2      |    1.8     |    0.73    |   mg/Kg    |        1         |  ☼ 6010C   |  Total/NA  |

### 🔍 Pattern Analysis for this Table:

✅ **No obviously misplaced qualifiers detected**

---


## 📋 Table 12 - Environmental Lab Data

> **⚠️ Document Convention Analysis:**
> - **U** = Undetected (below detection limit) - should be in separate qualifier column
> - **J** = Estimated value (detected but below reporting limit) - should be in separate qualifier column
> - **Expected Pattern:** Concentration | Qualifier | Reporting Limit | Method Detection Limit
> - **Look for:** Values like '0.046 U' that should be split into '0.046' and 'U'

|        Analyte         | Result Qualifier |     RL     |    MDL     |    Unit    | Dil Fac D Method | Prep Type  |
|:----------------------:|:----------------:|:----------:|:----------:|:----------:|:----------------:|:----------:|
|      Ethylbenzene      |       140        |  ⚠️ **J**  |    550     |     44     |      ug/Kg       |     50     |     ☼      |   8260C    |  Total/NA  |
| 1,3,5-Trimethylbenzene |       1100       |    550     |     55     |   ug/Kg    |        50        |     ☼      |   8260C    |  Total/NA  | ❌ *EMPTY*  |
|        Toluene         |       110        |  ⚠️ **J**  |    550     |     65     |      ug/Kg       |     50     |     ☼      |   8260C    |  Total/NA  |
|     Xylenes, Total     |       470        |  ⚠️ **J**  |    1100    |    150     |      ug/Kg       |     50     |     ☼      |   8260C    |  Total/NA  |
|      Naphthalene       |       290        |  ⚠️ **J**  |    550     |    220     |      ug/Kg       |     50     |     ☼      |   8260C    |  Total/NA  |
| 1,2,4-Trimethylbenzene |       2200       |    550     |     55     |   ug/Kg    |        50        |     ☼      |   8260C    |  Total/NA  | ❌ *EMPTY*  |
|    Isopropylbenzene    |       180        |  ⚠️ **J**  |    550     |     44     |      ug/Kg       |     50     |     ☼      |   8260C    |  Total/NA  |
|       Anthracene       |        31        |     28     |    5.5     |   ug/Kg    |        1         |     ☼      |   8270D    |  Total/NA  | ❌ *EMPTY*  |
|   Benzo[a]anthracene   |        22        |  ⚠️ **J**  |     28     |    5.5     |      ug/Kg       |     1      |     ☼      |   8270D    |  Total/NA  |
|     Benzo[a]pyrene     |        21        |  ⚠️ **J**  |     28     |    5.5     |      ug/Kg       |     1      |     ☼      |   8270D    |  Total/NA  |
|  Benzo[b]fluoranthene  |        40        |     28     |    5.5     |   ug/Kg    |        1         |     ☼      |   8270D    |  Total/NA  | ❌ *EMPTY*  |

### 🔍 Pattern Analysis for this Table:

✅ **No obviously misplaced qualifiers detected**
⚠️ **7 standalone qualifiers found** (verify they're in correct columns)

---


## 📋 Table 13 - Environmental Lab Data

> **⚠️ Document Convention Analysis:**
> - **U** = Undetected (below detection limit) - should be in separate qualifier column
> - **J** = Estimated value (detected but below reporting limit) - should be in separate qualifier column
> - **Expected Pattern:** Concentration | Qualifier | Reporting Limit | Method Detection Limit
> - **Look for:** Values like '0.046 U' that should be split into '0.046' and 'U'

|       Analyte        | Result Qualifier |     RL     |  MDL Unit  | Dil Fac D Method | Prep Type  |
|:--------------------:|:----------------:|:----------:|:----------:|:----------------:|:----------:|
| Benzo[g,h,i]perylene |        56        |     28     | ug/Kg 5.5  |       ☼ 1        |   8270D    |  Total/NA  |
|       Chrysene       |        30        |     28     | ug/Kg 5.5  |        1         |  ☼ 8270D   |  Total/NA  |
|     Phenanthrene     |        64        |     28     | ug/Kg 6.6  |       ☼ 1        |   8270D    |  Total/NA  |
|        Pyrene        |        54        |     28     | ug/Kg 5.5  |       ☼ 1        |   8270D    |  Total/NA  |
|         Lead         |        94        |    1.9     | mg/Kg 0.74 |       ☼ 1        |   6010C    |  Total/NA  |

### 🔍 Pattern Analysis for this Table:

✅ **No obviously misplaced qualifiers detected**

---


## 📋 Table 14 - Environmental Lab Data

> **⚠️ Document Convention Analysis:**
> - **U** = Undetected (below detection limit) - should be in separate qualifier column
> - **J** = Estimated value (detected but below reporting limit) - should be in separate qualifier column
> - **Expected Pattern:** Concentration | Qualifier | Reporting Limit | Method Detection Limit
> - **Look for:** Values like '0.046 U' that should be split into '0.046' and 'U'

|           Analyte           | Result Qualifier |     RL     |    MDL     |    Unit    | Dil Fac D Method | Prep Type  |
|:---------------------------:|:----------------:|:----------:|:----------:|:----------:|:----------------:|:----------:|
|        Ethylbenzene         |       1700       |    250     |     20     |   ug/Kg    |        50        |     ☼      |   8260C    |  Total/NA  | ❌ *EMPTY*  |
|   1,3,5-Trimethylbenzene    |        39        |  ⚠️ **J**  |    250     |     25     |      ug/Kg       |     50     |     ☼      |   8260C    |  Total/NA  |
|       Xylenes, Total        |       110        |  ⚠️ **J**  |    510     |     71     |      ug/Kg       |     50     |     ☼      |   8260C    |  Total/NA  |
|           Benzene           |        26        |  ⚠️ **J**  |    250     |     25     |      ug/Kg       |     50     |     ☼      |   8260C    |  Total/NA  |
|         Naphthalene         |       820        |    250     |    100     |   ug/Kg    |        50        |     ☼      |   8260C    |  Total/NA  | ❌ *EMPTY*  |
|      Isopropylbenzene       |       1000       |    250     |     20     |   ug/Kg    |        50        |     ☼      |   8260C    |  Total/NA  | ❌ *EMPTY*  |
| 1,2,4-Trimethylbenzene - DL |      27000       |    2500    |    250     |   ug/Kg    |       500        |     ☼      |   8260C    |  Total/NA  | ❌ *EMPTY*  |
|         Anthracene          |        35        |     18     |    3.7     |   ug/Kg    |        1         |     ☼      |   8270D    |  Total/NA  | ❌ *EMPTY*  |
|     Benzo[a]anthracene      |        62        |     18     |    3.7     |   ug/Kg    |        1         |     ☼      |   8270D    |  Total/NA  | ❌ *EMPTY*  |
|       Benzo[a]pyrene        |        47        |     18     |    3.7     |   ug/Kg    |        1         |     ☼      |   8270D    |  Total/NA  | ❌ *EMPTY*  |
|    Benzo[b]fluoranthene     |        44        |     18     |    3.7     |   ug/Kg    |        1         |     ☼      |   8270D    |  Total/NA  | ❌ *EMPTY*  |
|    Benzo[g,h,i]perylene     |        41        |     18     |    3.7     |   ug/Kg    |        1         |     ☼      |   8270D    |  Total/NA  | ❌ *EMPTY*  |
|          Chrysene           |        80        |     18     |    3.7     |   ug/Kg    |        1         |     ☼      |   8270D    |  Total/NA  | ❌ *EMPTY*  |
|          Fluorene           |       130        |     18     |    3.7     |   ug/Kg    |        1         |     ☼      |   8270D    |  Total/NA  | ❌ *EMPTY*  |
|        Phenanthrene         |       190        |     18     |    4.4     |   ug/Kg    |        1         |     ☼      |   8270D    |  Total/NA  | ❌ *EMPTY*  |
|           Pyrene            |       100        |     18     |    3.7     |   ug/Kg    |        1         |     ☼      |   8270D    |  Total/NA  | ❌ *EMPTY*  |
|            Lead             |        14        |    1.3     |    0.51    |   mg/Kg    |        1         |     ☼      |   6010C    |  Total/NA  | ❌ *EMPTY*  |

### 🔍 Pattern Analysis for this Table:

✅ **No obviously misplaced qualifiers detected**
⚠️ **3 standalone qualifiers found** (verify they're in correct columns)

---


## 📋 Table 15 - Environmental Lab Data

> **⚠️ Document Convention Analysis:**
> - **U** = Undetected (below detection limit) - should be in separate qualifier column
> - **J** = Estimated value (detected but below reporting limit) - should be in separate qualifier column
> - **Expected Pattern:** Concentration | Qualifier | Reporting Limit | Method Detection Limit
> - **Look for:** Values like '0.046 U' that should be split into '0.046' and 'U'

|        Analyte         | Result Qualifier |     RL     |    MDL     |    Unit    | Dil Fac D Method | Prep Type  |
|:----------------------:|:----------------:|:----------:|:----------:|:----------:|:----------------:|:----------:|
|      Ethylbenzene      |        42        |  ⚠️ **J**  |    340     |     27     |      ug/Kg       |     50     |     ☼      |   8260C    |  Total/NA  |
| 1,3,5-Trimethylbenzene |       180        |  ⚠️ **J**  |    340     |     34     |      ug/Kg       |     50     |     ☼      |   8260C    |  Total/NA  |
|        Toluene         |       220        |  ⚠️ **J**  |    340     |     41     |      ug/Kg       |     50     |     ☼      |   8260C    |  Total/NA  |
|     Xylenes, Total     |       300        |  ⚠️ **J**  |    680     |     95     |      ug/Kg       |     50     |     ☼      |   8260C    |  Total/NA  |
| 1,2,4-Trimethylbenzene |       420        |    340     |     34     |   ug/Kg    |        50        |     ☼      |   8260C    |  Total/NA  | ❌ *EMPTY*  |
|       Anthracene       |        18        |  ⚠️ **J**  |     20     |     4      |      ug/Kg       |     1      |     ☼      |   8270D    |  Total/NA  |
|   Benzo[a]anthracene   |        26        |     20     |     4      |   ug/Kg    |        1         |     ☼      |   8270D    |  Total/NA  | ❌ *EMPTY*  |
|     Benzo[a]pyrene     |        39        |     20     |     4      |   ug/Kg    |        1         |     ☼      |   8270D    |  Total/NA  | ❌ *EMPTY*  |
|  Benzo[b]fluoranthene  |        48        |     20     |     4      |   ug/Kg    |        1         |     ☼      |   8270D    |  Total/NA  | ❌ *EMPTY*  |
|  Benzo[g,h,i]perylene  |        95        |     20     |     4      |   ug/Kg    |        1         |     ☼      |   8270D    |  Total/NA  | ❌ *EMPTY*  |
|        Chrysene        |        34        |     20     |     4      |   ug/Kg    |        1         |     ☼      |   8270D    |  Total/NA  | ❌ *EMPTY*  |
|      Phenanthrene      |        35        |     20     |    4.8     |   ug/Kg    |        1         |     ☼      |   8270D    |  Total/NA  | ❌ *EMPTY*  |
|         Pyrene         |        44        |     20     |     4      |   ug/Kg    |        1         |     ☼      |   8270D    |  Total/NA  | ❌ *EMPTY*  |
|          Lead          |        18        |    1.6     |    0.65    |   mg/Kg    |        1         |     ☼      |   6010C    |  Total/NA  | ❌ *EMPTY*  |

### 🔍 Pattern Analysis for this Table:

✅ **No obviously misplaced qualifiers detected**
⚠️ **5 standalone qualifiers found** (verify they're in correct columns)

---


## 📋 Table 16 - Environmental Lab Data

> **⚠️ Document Convention Analysis:**
> - **U** = Undetected (below detection limit) - should be in separate qualifier column
> - **J** = Estimated value (detected but below reporting limit) - should be in separate qualifier column
> - **Expected Pattern:** Concentration | Qualifier | Reporting Limit | Method Detection Limit
> - **Look for:** Values like '0.046 U' that should be split into '0.046' and 'U'

|        Analyte         | Result Qualifier |     RL     |  MDL Unit  | Dil Fac D Method | Prep Type  |
|:----------------------:|:----------------:|:----------:|:----------:|:----------------:|:----------:|
|      Ethylbenzene      |        1         |  ⚠️ **J**  |    7.3     |    ug/Kg 0.58    |     1      |  ☼ 8260C   |  Total/NA  |
| 1,3,5-Trimethylbenzene |       0.73       |  ⚠️ **J**  |    7.3     |    ug/Kg 0.73    |    ☼ 1     |   8260C    |  Total/NA  |
|        Toluene         |       6.8        |  ⚠️ **J**  |    7.3     |    ug/Kg 0.87    |    ☼ 1     |   8260C    |  Total/NA  |
|     Xylenes, Total     |       5.9        |  ⚠️ **J**  |     15     |    ug/Kg 2.0     |    ☼ 1     |   8260C    |  Total/NA  |

### 🔍 Pattern Analysis for this Table:

✅ **No obviously misplaced qualifiers detected**
⚠️ **4 standalone qualifiers found** (verify they're in correct columns)

---


## 📋 Table 17 - Environmental Lab Data

> **⚠️ Document Convention Analysis:**
> - **U** = Undetected (below detection limit) - should be in separate qualifier column
> - **J** = Estimated value (detected but below reporting limit) - should be in separate qualifier column
> - **Expected Pattern:** Concentration | Qualifier | Reporting Limit | Method Detection Limit
> - **Look for:** Values like '0.046 U' that should be split into '0.046' and 'U'

|        Analyte         | Result Qualifier |     RL     |    MDL     |    Unit    | Dil Fac D Method | Prep Type  |
|:----------------------:|:----------------:|:----------:|:----------:|:----------:|:----------------:|:----------:|
|        Benzene         |       6.2        |  ⚠️ **J**  |    7.3     |    0.73    |      ug/Kg       |     1      |  ☼ 8260C   |  Total/NA  |
| 1,2,4-Trimethylbenzene |       1.2        |  ⚠️ **J**  |    7.3     |    0.73    |      ug/Kg       |     1      |  ☼ 8260C   |  Total/NA  |
|       Anthracene       |       110        |     24     |    4.7     |   ug/Kg    |        1         |  ☼ 8270D   |  Total/NA  | ❌ *EMPTY*  |
|   Benzo[a]anthracene   |       360        |     24     |    4.7     |   ug/Kg    |        1         |  ☼ 8270D   |  Total/NA  | ❌ *EMPTY*  |
|     Benzo[a]pyrene     |       390        |     24     |    4.7     |   ug/Kg    |        1         |  ☼ 8270D   |  Total/NA  | ❌ *EMPTY*  |
|  Benzo[b]fluoranthene  |       510        |     24     |    4.7     |   ug/Kg    |        1         |  ☼ 8270D   |  Total/NA  | ❌ *EMPTY*  |
|  Benzo[g,h,i]perylene  |       390        |     24     |    4.7     |   ug/Kg    |        1         |  ☼ 8270D   |  Total/NA  | ❌ *EMPTY*  |
|        Chrysene        |       390        |     24     |    4.7     |   ug/Kg    |        1         |  ☼ 8270D   |  Total/NA  | ❌ *EMPTY*  |
|        Fluorene        |        32        |     24     |    4.7     |   ug/Kg    |        1         |  ☼ 8270D   |  Total/NA  | ❌ *EMPTY*  |
|      Phenanthrene      |       450        |     24     |    5.7     |   ug/Kg    |        1         |  ☼ 8270D   |  Total/NA  | ❌ *EMPTY*  |
|         Pyrene         |       570        |     24     |    4.7     |   ug/Kg    |        1         |  ☼ 8270D   |  Total/NA  | ❌ *EMPTY*  |
|          Lead          |       200        |    1.9     |    0.78    |   mg/Kg    |        1         |  ☼ 6010C   |  Total/NA  | ❌ *EMPTY*  |

### 🔍 Pattern Analysis for this Table:

✅ **No obviously misplaced qualifiers detected**
⚠️ **2 standalone qualifiers found** (verify they're in correct columns)

---


## 📋 Table 18 - Environmental Lab Data

> **⚠️ Document Convention Analysis:**
> - **U** = Undetected (below detection limit) - should be in separate qualifier column
> - **J** = Estimated value (detected but below reporting limit) - should be in separate qualifier column
> - **Expected Pattern:** Concentration | Qualifier | Reporting Limit | Method Detection Limit
> - **Look for:** Values like '0.046 U' that should be split into '0.046' and 'U'

|           Analyte           | Result Qualifier |     RL     |    MDL     |    Unit    | Dil Fac D Method | Prep Type  |
|:---------------------------:|:----------------:|:----------:|:----------:|:----------:|:----------------:|:----------:|
|        Ethylbenzene         |       4100       |    460     |     37     |   ug/Kg    |        50        |     ☼      |   8260C    |  Total/NA  | ❌ *EMPTY*  |
|   1,3,5-Trimethylbenzene    |      15000       |    460     |     46     |   ug/Kg    |        50        |     ☼      |   8260C    |  Total/NA  | ❌ *EMPTY*  |
|           Toluene           |       8400       |    460     |     55     |   ug/Kg    |        50        |     ☼      |   8260C    |  Total/NA  | ❌ *EMPTY*  |
|       Xylenes, Total        |      31000       |    920     |    130     |   ug/Kg    |        50        |     ☼      |   8260C    |  Total/NA  | ❌ *EMPTY*  |
|           Benzene           |       720        |    460     |     46     |   ug/Kg    |        50        |     ☼      |   8260C    |  Total/NA  | ❌ *EMPTY*  |
|         Naphthalene         |       1100       |    460     |    180     |   ug/Kg    |        50        |     ☼      |   8260C    |  Total/NA  | ❌ *EMPTY*  |
|      Isopropylbenzene       |       2000       |    460     |     37     |   ug/Kg    |        50        |     ☼      |   8260C    |  Total/NA  | ❌ *EMPTY*  |
| 1,2,4-Trimethylbenzene - DL |      51000       |    4600    |    460     |   ug/Kg    |       500        |     ☼      |   8260C    |  Total/NA  | ❌ *EMPTY*  |
|         Anthracene          |        21        |  ⚠️ **J**  |     25     |     5      |      ug/Kg       |     1      |     ☼      |   8270D    |  Total/NA  |
|     Benzo[a]anthracene      |        26        |     25     |     5      |   ug/Kg    |        1         |     ☼      |   8270D    |  Total/NA  | ❌ *EMPTY*  |
|       Benzo[a]pyrene        |        17        |  ⚠️ **J**  |     25     |     5      |      ug/Kg       |     1      |     ☼      |   8270D    |  Total/NA  |
|    Benzo[b]fluoranthene     |        29        |     25     |     5      |   ug/Kg    |        1         |     ☼      |   8270D    |  Total/NA  | ❌ *EMPTY*  |
|    Benzo[g,h,i]perylene     |        43        |     25     |     5      |   ug/Kg    |        1         |     ☼      |   8270D    |  Total/NA  | ❌ *EMPTY*  |
|          Chrysene           |        63        |     25     |     5      |   ug/Kg    |        1         |     ☼      |   8270D    |  Total/NA  | ❌ *EMPTY*  |
|          Fluorene           |        31        |     25     |     5      |   ug/Kg    |        1         |     ☼      |   8270D    |  Total/NA  | ❌ *EMPTY*  |
|        Phenanthrene         |        74        |     25     |     6      |   ug/Kg    |        1         |     ☼      |   8270D    |  Total/NA  | ❌ *EMPTY*  |
|           Pyrene            |        36        |     25     |     5      |   ug/Kg    |        1         |     ☼      |   8270D    |  Total/NA  | ❌ *EMPTY*  |
|            Lead             |        21        |    2.3     |    0.9     |   mg/Kg    |        1         |     ☼      |   6010C    |  Total/NA  | ❌ *EMPTY*  |

### 🔍 Pattern Analysis for this Table:

✅ **No obviously misplaced qualifiers detected**
⚠️ **2 standalone qualifiers found** (verify they're in correct columns)

---


## 📋 Table 19 - Environmental Lab Data

> **⚠️ Document Convention Analysis:**
> - **U** = Undetected (below detection limit) - should be in separate qualifier column
> - **J** = Estimated value (detected but below reporting limit) - should be in separate qualifier column
> - **Expected Pattern:** Concentration | Qualifier | Reporting Limit | Method Detection Limit
> - **Look for:** Values like '0.046 U' that should be split into '0.046' and 'U'

|        Analyte         | Result Qualifier |     RL     |    MDL     |    Unit    | Dil Fac D Method | Prep Type  |
|:----------------------:|:----------------:|:----------:|:----------:|:----------:|:----------------:|:----------:|
|      Ethylbenzene      |       1.4        |  ⚠️ **J**  |    5.6     |    0.44    |      ug/Kg       |     1      |  ☼ 8260C   |  Total/NA  |
|   1,2-Dichloroethane   |       1.3        |  ⚠️ **J**  |    5.6     |    0.67    |      ug/Kg       |     1      |  ☼ 8260C   |  Total/NA  |
|        Toluene         |        31        |    5.6     |    0.67    |   ug/Kg    |        1         |  ☼ 8260C   |  Total/NA  | ❌ *EMPTY*  |
|     Xylenes, Total     |       7.3        |  ⚠️ **J**  |     11     |    1.6     |      ug/Kg       |     1      |  ☼ 8260C   |  Total/NA  |
|        Benzene         |        12        |    5.6     |    0.56    |   ug/Kg    |        1         |  ☼ 8260C   |  Total/NA  | ❌ *EMPTY*  |
| 1,2,4-Trimethylbenzene |       1.1        |  ⚠️ **J**  |    5.6     |    0.56    |      ug/Kg       |     1      |  ☼ 8260C   |  Total/NA  |
|       Anthracene       |        48        |     20     |    4.1     |   ug/Kg    |        1         |  ☼ 8270D   |  Total/NA  | ❌ *EMPTY*  |
|   Benzo[a]anthracene   |       120        |     20     |    4.1     |   ug/Kg    |        1         |  ☼ 8270D   |  Total/NA  | ❌ *EMPTY*  |
|     Benzo[a]pyrene     |       330        |     20     |    4.1     |   ug/Kg    |        1         |  ☼ 8270D   |  Total/NA  | ❌ *EMPTY*  |
|  Benzo[b]fluoranthene  |       370        |     20     |    4.1     |   ug/Kg    |        1         |  ☼ 8270D   |  Total/NA  | ❌ *EMPTY*  |
|  Benzo[g,h,i]perylene  |       430        |     20     |    4.1     |   ug/Kg    |        1         |  ☼ 8270D   |  Total/NA  | ❌ *EMPTY*  |
|        Chrysene        |       130        |     20     |    4.1     |   ug/Kg    |        1         |  ☼ 8270D   |  Total/NA  | ❌ *EMPTY*  |
|        Fluorene        |        12        |  ⚠️ **J**  |     20     |    4.1     |      ug/Kg       |     1      |  ☼ 8270D   |  Total/NA  |
|      Phenanthrene      |        97        |     20     |    4.9     |   ug/Kg    |        1         |  ☼ 8270D   |  Total/NA  | ❌ *EMPTY*  |

### 🔍 Pattern Analysis for this Table:

✅ **No obviously misplaced qualifiers detected**
⚠️ **5 standalone qualifiers found** (verify they're in correct columns)

---


## 📋 Table 20 - Environmental Lab Data

> **⚠️ Document Convention Analysis:**
> - **U** = Undetected (below detection limit) - should be in separate qualifier column
> - **J** = Estimated value (detected but below reporting limit) - should be in separate qualifier column
> - **Expected Pattern:** Concentration | Qualifier | Reporting Limit | Method Detection Limit
> - **Look for:** Values like '0.046 U' that should be split into '0.046' and 'U'

|  Analyte   | Result Qualifier |     RL     |  MDL Unit  | Dil Fac D Method | Prep Type  |
|:----------:|:----------------:|:----------:|:----------:|:----------------:|:----------:|
|   Pyrene   |       140        |     20     | ug/Kg 4.1  |    ☼ 1 8270D     |  Total/NA  |
|    Lead    |       500        |    1.4     | mg/Kg 0.56 |    ☼ 1 6010C     |  Total/NA  |

### 🔍 Pattern Analysis for this Table:

✅ **No obviously misplaced qualifiers detected**

---


## 📋 Table 21 - Environmental Lab Data

> **⚠️ Document Convention Analysis:**
> - **U** = Undetected (below detection limit) - should be in separate qualifier column
> - **J** = Estimated value (detected but below reporting limit) - should be in separate qualifier column
> - **Expected Pattern:** Concentration | Qualifier | Reporting Limit | Method Detection Limit
> - **Look for:** Values like '0.046 U' that should be split into '0.046' and 'U'

|        Analyte         | Result Qualifier |     RL     |    MDL     |    Unit    | Dil Fac D Method | Prep Type  |
|:----------------------:|:----------------:|:----------:|:----------:|:----------:|:----------------:|:----------:|
|        Toluene         |       5.5        |    5.4     |    0.65    |   ug/Kg    |       ☼ 1        |   8260C    |  Total/NA  | ❌ *EMPTY*  |
|     Xylenes, Total     |       2.8        |  ⚠️ **J**  |     11     |    1.5     |      ug/Kg       |    ☼ 1     |   8260C    |  Total/NA  |
|        Benzene         |       3.8        |  ⚠️ **J**  |    5.4     |    0.54    |      ug/Kg       |    ☼ 1     |   8260C    |  Total/NA  |
| 1,2,4-Trimethylbenzene |       0.58       |  ⚠️ **J**  |    5.4     |    0.54    |      ug/Kg       |    ☼ 1     |   8260C    |  Total/NA  |
|       Anthracene       |        55        |     20     |     4      |   ug/Kg    |        1         |  ☼ 8270D   |  Total/NA  | ❌ *EMPTY*  |
|   Benzo[a]anthracene   |       350        |     20     |     4      |   ug/Kg    |        1         |  ☼ 8270D   |  Total/NA  | ❌ *EMPTY*  |
|     Benzo[a]pyrene     |       330        |     20     |     4      |   ug/Kg    |        1         |  ☼ 8270D   |  Total/NA  | ❌ *EMPTY*  |
|  Benzo[b]fluoranthene  |       490        |     20     |     4      |   ug/Kg    |       ☼ 1        |   8270D    |  Total/NA  | ❌ *EMPTY*  |
|  Benzo[g,h,i]perylene  |       300        |     20     |     4      |   ug/Kg    |       ☼ 1        |   8270D    |  Total/NA  | ❌ *EMPTY*  |
|        Chrysene        |       370        |     20     |     4      |   ug/Kg    |        1         |  ☼ 8270D   |  Total/NA  | ❌ *EMPTY*  |
|        Fluorene        |        15        |  ⚠️ **J**  |     20     |     4      |      ug/Kg       |    ☼ 1     |   8270D    |  Total/NA  |
|      Phenanthrene      |       190        |     20     |    4.8     |   ug/Kg    |        1         |  ☼ 8270D   |  Total/NA  | ❌ *EMPTY*  |
|         Pyrene         |       560        |     20     |     4      |   ug/Kg    |        1         |  ☼ 8270D   |  Total/NA  | ❌ *EMPTY*  |
|          Lead          |        76        |    1.2     |    0.5     |   mg/Kg    |       ☼ 1        |   6010C    |  Total/NA  | ❌ *EMPTY*  |

### 🔍 Pattern Analysis for this Table:

✅ **No obviously misplaced qualifiers detected**
⚠️ **4 standalone qualifiers found** (verify they're in correct columns)

---


## 📋 Table 22 - Environmental Lab Data

> **⚠️ Document Convention Analysis:**
> - **U** = Undetected (below detection limit) - should be in separate qualifier column
> - **J** = Estimated value (detected but below reporting limit) - should be in separate qualifier column
> - **Expected Pattern:** Concentration | Qualifier | Reporting Limit | Method Detection Limit
> - **Look for:** Values like '0.046 U' that should be split into '0.046' and 'U'

|        Analyte         | Result Qualifier |     RL     |    MDL     |    Unit    | Dil Fac D Method | Prep Type  |
|:----------------------:|:----------------:|:----------:|:----------:|:----------:|:----------------:|:----------:|
|      Ethylbenzene      |       3.8        |  ⚠️ **J**  |    5.9     |    0.47    |      ug/Kg       |     1      |  ☼ 8260C   |  Total/NA  |
| 1,3,5-Trimethylbenzene |       1.4        |  ⚠️ **J**  |    5.9     |    0.59    |      ug/Kg       |     1      |  ☼ 8260C   |  Total/NA  |
|        Toluene         |        11        |    5.9     |    0.71    |   ug/Kg    |        1         |  ☼ 8260C   |  Total/NA  | ❌ *EMPTY*  |
|     Xylenes, Total     |        12        |     12     |    1.7     |   ug/Kg    |        1         |  ☼ 8260C   |  Total/NA  | ❌ *EMPTY*  |
|        Benzene         |       3.8        |  ⚠️ **J**  |    5.9     |    0.59    |      ug/Kg       |     1      |  ☼ 8260C   |  Total/NA  |
| 1,2,4-Trimethylbenzene |       5.7        |  ⚠️ **J**  |    5.9     |    0.59    |      ug/Kg       |     1      |  ☼ 8260C   |  Total/NA  |
|       Anthracene       |        33        |     19     |    3.8     |   ug/Kg    |        1         |  ☼ 8270D   |  Total/NA  | ❌ *EMPTY*  |
|   Benzo[a]anthracene   |        57        |     19     |    3.8     |   ug/Kg    |        1         |  ☼ 8270D   |  Total/NA  | ❌ *EMPTY*  |
|     Benzo[a]pyrene     |        94        |     19     |    3.8     |   ug/Kg    |        1         |  ☼ 8270D   |  Total/NA  | ❌ *EMPTY*  |
|  Benzo[b]fluoranthene  |       100        |     19     |    3.8     |   ug/Kg    |        1         |  ☼ 8270D   |  Total/NA  | ❌ *EMPTY*  |
|  Benzo[g,h,i]perylene  |       110        |     19     |    3.8     |   ug/Kg    |        1         |  ☼ 8270D   |  Total/NA  | ❌ *EMPTY*  |
|        Chrysene        |        65        |     19     |    3.8     |   ug/Kg    |        1         |  ☼ 8270D   |  Total/NA  | ❌ *EMPTY*  |
|        Fluorene        |        21        |     19     |    3.8     |   ug/Kg    |        1         |  ☼ 8270D   |  Total/NA  | ❌ *EMPTY*  |
|      Phenanthrene      |       110        |     19     |    4.6     |   ug/Kg    |        1         |  ☼ 8270D   |  Total/NA  | ❌ *EMPTY*  |
|         Pyrene         |       120        |     19     |    3.8     |   ug/Kg    |        1         |  ☼ 8270D   |  Total/NA  | ❌ *EMPTY*  |
|          Lead          |        30        |    1.4     |    0.56    |   mg/Kg    |        1         |  ☼ 6010C   |  Total/NA  | ❌ *EMPTY*  |

### 🔍 Pattern Analysis for this Table:

✅ **No obviously misplaced qualifiers detected**
⚠️ **4 standalone qualifiers found** (verify they're in correct columns)

---


## 📋 Table 23 - Environmental Lab Data

> **⚠️ Document Convention Analysis:**
> - **U** = Undetected (below detection limit) - should be in separate qualifier column
> - **J** = Estimated value (detected but below reporting limit) - should be in separate qualifier column
> - **Expected Pattern:** Concentration | Qualifier | Reporting Limit | Method Detection Limit
> - **Look for:** Values like '0.046 U' that should be split into '0.046' and 'U'

|       Analyte        | Result Qualifier |     RL     |  MDL Unit  | Dil Fac D Method | Prep Type  |
|:--------------------:|:----------------:|:----------:|:----------:|:----------------:|:----------:|
|  Benzo[a]anthracene  |      J 9.8       |     21     | ug/Kg 4.1  |       ☼ 1        |   8270D    |  Total/NA  |
|    Benzo[a]pyrene    |  🔴 **6.2 J** 🔴   |     21     | ug/Kg 4.1  |       ☼ 1        |   8270D    |  Total/NA  |
| Benzo[b]fluoranthene |  🔴 **9.9 J** 🔴   |     21     | ug/Kg 4.1  |       ☼ 1        |   8270D    |  Total/NA  |
| Benzo[g,h,i]perylene |   🔴 **11 J** 🔴   |     21     | ug/Kg 4.1  |       ☼ 1        |   8270D    |  Total/NA  |
|       Chrysene       |  🔴 **9.7 J** 🔴   |     21     | ug/Kg 4.1  |       ☼ 1        |   8270D    |  Total/NA  |
|     Phenanthrene     |   🔴 **10 J** 🔴   |     21     | ug/Kg 5.0  |       ☼ 1        |   8270D    |  Total/NA  |
|        Pyrene        |   🔴 **13 J** 🔴   |     21     | ug/Kg 4.1  |       ☼ 1        |   8270D    |  Total/NA  |
|         Lead         |       320        |    1.7     | mg/Kg 0.69 |       ☼ 1        |   6010C    |  Total/NA  |

### 🔍 Pattern Analysis for this Table:

**❌ 6 misplaced qualifiers found!**
- 🔴 **Misplaced qualifier detected:** `6.2 J` (should be split)
- 🔴 **Misplaced qualifier detected:** `9.9 J` (should be split)
- 🔴 **Misplaced qualifier detected:** `11 J` (should be split)
- 🔴 **Misplaced qualifier detected:** `9.7 J` (should be split)
- 🔴 **Misplaced qualifier detected:** `10 J` (should be split)
- ... and 1 more

---

## ✅ Quality Control Validation Checklist

Use this checklist while comparing against the original PDF:

- [ ] **Headers:** Match original PDF table structure exactly
- [ ] **Qualifiers:** All 🔴 marked items are extraction errors needing fixing
- [ ] **Numeric alignment:** Values didn't shift between columns
- [ ] **Missing data:** No ❌ *EMPTY* cells where data should exist
- [ ] **Column patterns:** Conc/Q/RL/MDL structure preserved where expected
- [ ] **Data integrity:** All lab qualifiers (U, J, etc.) in proper qualifier columns

---

*This report was generated automatically. For compliance reporting, manual validation against the original PDF is required.*
