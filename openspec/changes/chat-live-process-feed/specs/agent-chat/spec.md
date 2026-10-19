## MODIFIED Requirements

### Requirement: Assistant Turn Trajectory Display
聊天界面 SHALL 将同一助手回合内的思考、工具调用与正文按发生顺序渲染为一条时间线（轨迹）。

流式进行中，过程区（思考与工具）MUST 默认展开，并以**一行简略摘要**逐条展示各步骤（随事件增量追加/更新）；步骤的参数、结果与推理全文 MUST 默认不展开，用户点击该摘要行后方可查看。正文 token MUST 仍在过程区下方展示。

回合结束后（及历史重开）过程区 MUST 默认收起为活动标题（例如「已完成分析」「已读取文件并搜索代码」），正文 MUST 保持展开。用户可手动展开过程区与单步细节；重开会话后 MUST 恢复上述默认折叠规则。进行中的回合 MUST 在正文下方显示「努力思考中」，满 1 秒后带上用时。

#### Scenario: Streaming turn shows ordered brief process lines
- **WHEN** 一轮包含思考与工具调用的回复正在流式生成
- **THEN** 过程区默认展开；标题为当前活动（如「正在分析请求」「正在读取文件」）并带扫光；思考行标题为「思考」，其右侧为最近一段的首行预览；参数/结果默认不可见；正文下方显示「努力思考中」

#### Scenario: Streaming process list stays compact
- **WHEN** 流式过程中步骤较多
- **THEN** 过程列表以有限高度展示并可滚动到最新步骤，MUST NOT 占满整个聊天视口

#### Scenario: Completed turn collapses process steps
- **WHEN** 该回合完成或用户重新打开含轨迹的历史会话
- **THEN** 过程区默认收起为活动标题，助手正文保持展开可见；展开后思考行可看到首行预览，点开后可见全文

#### Scenario: Manual expand does not persist across reopen
- **WHEN** 用户在已完成回合中手动展开过程区或某步细节后关闭并重新打开该会话
- **THEN** 该回合再次以默认收起状态展示
