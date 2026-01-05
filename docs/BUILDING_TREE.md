# 宗门建筑树系统使用指南

## 概述

宗门建筑树系统是一个树状的建筑科技系统，具有以下特性：

1. **树状依赖**：只有一个根节点，子建筑只能在父建筑建造后才能建造
2. **成本倍增**：每建造一个建筑，下一个建筑的成本翻倍（2^n倍）
3. **条件加成**：每个建筑提供一组条件modifier，只对满足条件的弟子生效

## 核心组件

### 1. Building - 建筑定义

```rust
pub struct Building {
    pub id: String,
    pub name: String,
    pub description: String,
    pub base_cost: u32,  // 基础建造成本
    pub parent_id: Option<String>,  // 父建筑ID，None表示根节点
    pub conditional_modifiers: Vec<ConditionalModifier>,  // 建筑提供的条件modifier
    pub is_built: bool,  // 是否已建造
}
```

### 2. BuildingTree - 建筑树

```rust
pub struct BuildingTree {
    pub buildings: HashMap<String, Building>,
    pub root_id: String,
    pub buildings_built_count: u32,  // 已建造的建筑数量（用于计算成本倍增）
}
```

### 3. 成本计算公式

```
实际成本 = 基础成本 × 2^已建造建筑数量
```

**示例**：
- 第1个建筑（基础成本100）：100 × 2^0 = 100
- 第2个建筑（基础成本200）：200 × 2^1 = 400
- 第3个建筑（基础成本150）：150 × 2^2 = 600
- 第4个建筑（基础成本300）：300 × 2^3 = 2400

## 使用方法

### 创建建筑

```rust
use crate::building::Building;
use crate::modifier::{Modifier, ModifierTarget, ModifierApplication, ModifierSource, ModifierCondition, ConditionalModifier};
use crate::disciple::DiscipleType;

// 1. 创建根建筑（宗门大殿）
let root_building = Building::new_root(
    "main_hall",
    "宗门大殿",
    "宗门的核心建筑，所有建筑的根基",
    100,  // 基础成本
    vec![
        ConditionalModifier::new(
            ModifierCondition::Always,  // 对所有弟子生效
            Modifier::new(
                "大殿庇护",
                ModifierTarget::DaoHeart,
                ModifierApplication::Additive(5.0),  // 所有弟子道心+5
                ModifierSource::Environment,
            )
        )
    ],
);

// 2. 创建子建筑（藏书楼）
let library = Building::new_child(
    "library",
    "藏书楼",
    "存放功法典籍的地方",
    200,
    "main_hall",  // 父建筑ID
    vec![
        ConditionalModifier::new(
            ModifierCondition::DiscipleTypeEquals(DiscipleType::Inner),
            Modifier::new(
                "典籍加持",
                ModifierTarget::TaskReward,
                ModifierApplication::Multiplicative(0.15),  // 内门弟子修炼+15%
                ModifierSource::Environment,
            )
        )
    ],
);

// 3. 创建更多子建筑
let training_ground = Building::new_child(
    "training",
    "练武场",
    "弟子修炼的场所",
    150,
    "main_hall",
    vec![
        ConditionalModifier::new(
            ModifierCondition::DaoHeartGreaterThan(50),
            Modifier::new(
                "武场磨砺",
                ModifierTarget::ConstitutionConsumption,
                ModifierApplication::Multiplicative(-0.1),  // 高道心弟子体魄消耗-10%
                ModifierSource::Environment,
            )
        )
    ],
);
```

### 构建建筑树

```rust
use crate::building::BuildingTree;

// 方法1：手动构建
let mut tree = BuildingTree::new(root_building);
tree.add_building(library).unwrap();
tree.add_building(training_ground).unwrap();

// 方法2：使用构建器（推荐）
use crate::building::BuildingTreeBuilder;

let tree = BuildingTreeBuilder::new()
    .root(root_building)
    .child(library)
    .child(training_ground)
    .build()
    .unwrap();
```

### 初始化宗门建筑树

```rust
// 在创建宗门后初始化建筑树
let mut sect = Sect::new("青云宗".to_string());
sect.init_building_tree(tree);
```

### 建造建筑

```rust
// 1. 查看可建造的建筑
let buildable = sect.get_buildable_buildings_with_cost();
for (id, name, cost) in buildable {
    println!("可建造：{} - {}，成本：{}", id, name, cost);
}

// 2. 建造建筑
match sect.build_building("main_hall") {
    Ok(msg) => println!("{}", msg),
    Err(e) => println!("建造失败：{}", e),
}

// 3. 查看建筑树状态
if let Some(summary) = sect.get_building_tree_summary() {
    println!("{}", summary);
}
```

## 完整示例：创建修仙宗门建筑树

```rust
use crate::building::{Building, BuildingTreeBuilder};
use crate::modifier::*;
use crate::cultivation::CultivationLevel;
use crate::disciple::{DiscipleType, TalentType};

fn create_xiuxian_building_tree() -> BuildingTree {
    // === 第一层：根节点 ===
    let main_hall = Building::new_root(
        "main_hall",
        "宗门大殿",
        "宗门的核心建筑",
        100,
        vec![
            ConditionalModifier::new(
                ModifierCondition::Always,
                Modifier::new("大殿庇护", ModifierTarget::DaoHeart,
                    ModifierApplication::Additive(5.0), ModifierSource::Environment)
            )
        ],
    );

    // === 第二层：基础建筑 ===

    // 藏书楼：提升内门弟子修炼速度
    let library = Building::new_child(
        "library",
        "藏书楼",
        "存放功法典籍",
        200,
        "main_hall",
        vec![
            ConditionalModifier::new(
                ModifierCondition::DiscipleTypeEquals(DiscipleType::Inner),
                Modifier::new("典籍加持", ModifierTarget::TaskReward,
                    ModifierApplication::Multiplicative(0.15), ModifierSource::Environment)
            )
        ],
    );

    // 练武场：降低高道心弟子的体魄消耗
    let training_ground = Building::new_child(
        "training",
        "练武场",
        "弟子修炼的场所",
        150,
        "main_hall",
        vec![
            ConditionalModifier::new(
                ModifierCondition::DaoHeartGreaterThan(50),
                Modifier::new("武场磨砺", ModifierTarget::ConstitutionConsumption,
                    ModifierApplication::Multiplicative(-0.1), ModifierSource::Environment)
            )
        ],
    );

    // 灵药园：降低所有弟子精力消耗
    let herb_garden = Building::new_child(
        "herb_garden",
        "灵药园",
        "种植灵药的园地",
        180,
        "main_hall",
        vec![
            ConditionalModifier::new(
                ModifierCondition::Always,
                Modifier::new("灵气滋养", ModifierTarget::EnergyConsumption,
                    ModifierApplication::Multiplicative(-0.05), ModifierSource::Environment)
            )
        ],
    );

    // === 第三层：高级建筑 ===

    // 传承殿：提升亲传弟子渡劫成功率（需要藏书楼）
    let heritage_hall = Building::new_child(
        "heritage_hall",
        "传承殿",
        "存放宗门至高传承",
        300,
        "library",
        vec![
            ConditionalModifier::new(
                ModifierCondition::DiscipleTypeEquals(DiscipleType::Personal),
                Modifier::new("传承庇护", ModifierTarget::TribulationSuccessRate,
                    ModifierApplication::Additive(0.1), ModifierSource::Heritage)
            )
        ],
    );

    // 剑冢：提升剑道天赋弟子战力（需要练武场）
    let sword_tomb = Building::new_child(
        "sword_tomb",
        "剑冢",
        "埋葬历代剑修之剑",
        250,
        "training",
        vec![
            ConditionalModifier::new(
                ModifierCondition::HasTalent(TalentType::Sword),
                Modifier::new("剑意共鸣", ModifierTarget::TalentBonus("Sword".to_string()),
                    ModifierApplication::Additive(0.3), ModifierSource::Heritage)
            )
        ],
    );

    // 炼丹房：提升炼丹天赋弟子能力（需要灵药园）
    let alchemy_room = Building::new_child(
        "alchemy",
        "炼丹房",
        "炼制丹药的场所",
        280,
        "herb_garden",
        vec![
            ConditionalModifier::new(
                ModifierCondition::HasTalent(TalentType::Alchemy),
                Modifier::new("丹火传承", ModifierTarget::TalentBonus("Alchemy".to_string()),
                    ModifierApplication::Additive(0.35), ModifierSource::Heritage)
            )
        ],
    );

    // === 第四层：终极建筑 ===

    // 飞升台：大幅提升高修为弟子渡劫率（需要传承殿）
    let ascension_platform = Building::new_child(
        "ascension",
        "飞升台",
        "通往仙界的门户",
        500,
        "heritage_hall",
        vec![
            ConditionalModifier::new(
                ModifierCondition::CultivationLevelGreaterOrEqual(CultivationLevel::VoidRefinement),
                Modifier::new("仙界感召", ModifierTarget::TribulationSuccessRate,
                    ModifierApplication::Additive(0.2), ModifierSource::System)
            )
        ],
    );

    // 无极剑阵：顶级剑修加成（需要剑冢）
    let sword_formation = Building::new_child(
        "sword_formation",
        "无极剑阵",
        "由千万神剑组成的剑阵",
        450,
        "sword_tomb",
        vec![
            ConditionalModifier::new(
                ModifierCondition::And(vec![
                    ModifierCondition::HasTalent(TalentType::Sword),
                    ModifierCondition::CultivationLevelGreaterOrEqual(CultivationLevel::GoldenCore),
                ]),
                Modifier::new("万剑归宗", ModifierTarget::TaskReward,
                    ModifierApplication::Multiplicative(0.4), ModifierSource::Heritage)
            )
        ],
    );

    // 构建建筑树
    BuildingTreeBuilder::new()
        // 第一层
        .root(main_hall)
        // 第二层
        .child(library)
        .child(training_ground)
        .child(herb_garden)
        // 第三层
        .child(heritage_hall)
        .child(sword_tomb)
        .child(alchemy_room)
        // 第四层
        .child(ascension_platform)
        .child(sword_formation)
        .build()
        .unwrap()
}
```

## 建筑树可视化

```
                    宗门大殿 (100)
                        |
            +-----------+-----------+
            |           |           |
        藏书楼(200)  练武场(150)  灵药园(180)
            |           |           |
        传承殿(300)  剑冢(250)  炼丹房(280)
            |           |
        飞升台(500)  剑阵(450)
```

**成本计算示例**：
1. 宗门大殿：100 × 2^0 = **100**
2. 藏书楼：200 × 2^1 = **400**
3. 练武场：150 × 2^2 = **600**
4. 灵药园：180 × 2^3 = **1440**
5. 传承殿：300 × 2^4 = **4800**
6. 剑冢：250 × 2^5 = **8000**
7. 炼丹房：280 × 2^6 = **17920**
8. 飞升台：500 × 2^7 = **64000**
9. 剑阵：450 × 2^8 = **115200**

**总成本**：212,460资源

## API参考

### Building方法

```rust
// 创建根建筑
Building::new_root(id, name, description, base_cost, modifiers)

// 创建子建筑
Building::new_child(id, name, description, base_cost, parent_id, modifiers)
```

### BuildingTree方法

```rust
// 添加建筑
tree.add_building(building) -> Result<(), String>

// 检查是否可以建造
tree.can_build(building_id) -> Result<(), String>

// 计算建造成本
tree.calculate_build_cost(building_id) -> Result<u32, String>

// 建造建筑
tree.build(building_id) -> Result<Vec<ConditionalModifier>, String>

// 获取所有已建造建筑的modifiers
tree.get_all_modifiers() -> Vec<ConditionalModifier>

// 获取可建造的建筑列表
tree.get_buildable_buildings() -> Vec<&Building>

// 获取子建筑
tree.get_children(parent_id) -> Vec<&Building>

// 获取建筑树深度
tree.get_depth() -> usize

// 获取已建造/总建筑数量
tree.get_built_count() -> usize
tree.get_total_count() -> usize
```

### Sect方法

```rust
// 初始化建筑树
sect.init_building_tree(tree)

// 建造建筑
sect.build_building(building_id) -> Result<String, String>

// 获取可建造建筑（含成本）
sect.get_buildable_buildings_with_cost() -> Vec<(String, String, u32)>

// 获取建筑树摘要
sect.get_building_tree_summary() -> Option<String>

// 获取对弟子生效的所有modifier（包括建筑提供的）
sect.get_applicable_modifiers_owned(disciple) -> Vec<Modifier>
```

## 游戏流程示例

```rust
// 1. 创建宗门和建筑树
let mut sect = Sect::new("青云宗".to_string());
let tree = create_xiuxian_building_tree();
sect.init_building_tree(tree);

// 2. 查看初始状态
println!("初始资源：{}", sect.resources);
println!("{}", sect.get_building_tree_summary().unwrap());

// 3. 建造第一个建筑（根节点）
match sect.build_building("main_hall") {
    Ok(msg) => println!("{}", msg),
    Err(e) => println!("失败：{}", e),
}

// 4. 查看可建造列表
println!("\n可建造的建筑：");
for (id, name, cost) in sect.get_buildable_buildings_with_cost() {
    println!("  {} - 成本：{}", name, cost);
}

// 5. 继续建造
sect.resources += 500;  // 假设获得了资源
sect.build_building("library").unwrap();
sect.build_building("training").unwrap();

// 6. 检查弟子获得的加成
let disciple = /* 某个内门弟子 */;
let applicable_mods = sect.get_applicable_modifiers_owned(&disciple);
println!("\n弟子{}获得{}个加成", disciple.name, applicable_mods.len());
```

## 设计考虑

### 成本倍增的意义

1. **早期选择重要**：前几个建筑成本较低，选择很重要
2. **后期极其昂贵**：建造所有建筑需要大量资源
3. **策略性选择**：玩家需要根据宗门发展方向选择建造路线

### 建筑效果设计原则

1. **条件明确**：每个建筑的生效条件应该清晰
2. **逐层递进**：后期建筑效果更强，但条件更苛刻
3. **路线差异**：不同建造路线适合不同的宗门发展策略

### 扩展性

系统支持轻松添加新建筑：
- 定义新Building
- 设置parent_id指向已有建筑
- 配置条件和modifier
- 添加到建筑树

## 注意事项

1. **建造顺序很重要**：成本倍增机制让建造顺序影响巨大
2. **合理规划**：建议先规划好完整的建造路线再执行
3. **资源管理**：确保有足够资源，否则建造失败不退还
4. **效果叠加**：建筑效果会与宗门modifier和个人modifier叠加

这个建筑树系统为宗门发展提供了丰富的策略深度！
