# Racing Game

红白机风格赛车游戏，使用 Rust + Bevy 开发。

## 运行

```bash
cd racing_game
cargo run
```

## 控制

- `←` / `A` - 左移
- `→` / `D` - 右移
- `Space` - 开始游戏

## 项目结构

```
src/
├── main.rs        # 入口和配置
├── game/
│   └── mod.rs     # 游戏状态管理
├── player/
│   └── mod.rs     # 玩家控制
├── road/
│   └── mod.rs     # 道路渲染
└── obstacle/
    └── mod.rs     # 障碍物系统
```

## 功能

- [x] 基础项目骨架
- [x] 菜单系统
- [x] 玩家移动
- [x] 道路滚动
- [x] 障碍物生成
- [x] 碰撞检测
- [ ] 分数系统
- [ ] 游戏结束界面
- [ ] 音效
- [ ] 美术素材
