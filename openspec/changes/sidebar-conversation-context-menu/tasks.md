## 1. SideBar 右键菜单壳

- [x] 1.1 移除会话行悬停删除按钮及相关样式
- [x] 1.2 为会话行绑定 `@contextmenu.prevent`，记录 id 与客户端坐标并打开浮层
- [x] 1.3 实现菜单关闭：点击外部、Esc、侧栏滚动、选中项后关闭；靠近视口边缘时翻转定位

## 2. 菜单动作

- [x] 2.1 「改名」：该行就地 input；Enter/失焦提交 `renameConversation`，Esc 取消；空标题不提交
- [x] 2.2 「打开轨迹」：`router.push({ path: \`/agent-runs/${id}\` })`
- [x] 2.3 「删除」：`confirm` 后调用 `deleteConversation`

## 3. 样式与验收

- [x] 3.1 菜单使用现有 shell token，危险项（删除）区分样式；暗色模式可读
- [x] 3.2 手动验收：右键三项可用、无悬停删除、左键仍打开会话
