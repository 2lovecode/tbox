// 拼音工具 - 支持中文转拼音、拼音首字母、模糊拼音匹配

// 常用汉字拼音映射表 (避免重复key)
const pinyinData: [string, string][] = [
  // 基础字符
  ['啊', 'a'], ['阿', 'a'], ['八', 'ba'], ['把', 'ba'], ['吧', 'ba'], ['拔', 'ba'], ['罢', 'ba'], ['霸', 'ba'], ['坝', 'ba'], ['爸', 'ba'],
  ['白', 'bai'], ['百', 'bai'], ['拜', 'bai'], ['办', 'ban'], ['半', 'ban'], ['班', 'ban'], ['板', 'ban'], ['版', 'ban'], ['扮', 'ban'], ['伴', 'ban'],
  ['被', 'bei'], ['北', 'bei'], ['备', 'bei'], ['背', 'bei'], ['倍', 'bei'],
  ['本', 'ben'], ['比', 'bi'], ['笔', 'bi'], ['必', 'bi'], ['闭', 'bi'], ['鼻', 'bi'],
  ['边', 'bian'], ['便', 'bian'], ['变', 'bian'], ['编', 'bian'],
  ['表', 'biao'],
  ['别', 'bie'],
  ['保', 'bao'], ['报', 'bao'], ['包', 'bao'], ['宝', 'bao'], ['抱', 'bao'], ['暴', 'bao'],
  ['布', 'bu'], ['部', 'bu'], ['不', 'bu'], ['步', 'bu'],
  ['才', 'cai'], ['菜', 'cai'], ['彩', 'cai'],
  ['苍', 'cang'], ['操', 'cao'], ['槽', 'cao'],
  ['测', 'ce'], ['层', 'ceng'], ['曾', 'ceng'],
  ['查', 'cha'], ['茶', 'cha'], ['差', 'cha'],
  ['产', 'chan'],
  ['常', 'chang'], ['场', 'chang'], ['长', 'chang'], ['唱', 'chang'],
  ['超', 'chao'], ['朝', 'chao'],
  ['车', 'che'], ['陈', 'chen'], ['称', 'cheng'], ['成', 'cheng'], ['城', 'cheng'], ['程', 'cheng'],
  ['吃', 'chi'], ['持', 'chi'], ['池', 'chi'], ['迟', 'chi'],
  ['出', 'chu'], ['初', 'chu'], ['除', 'chu'], ['处', 'chu'], ['楚', 'chu'], ['础', 'chu'], ['储', 'chu'],
  ['穿', 'chuan'], ['传', 'chuan'],
  ['创', 'chuang'],
  ['春', 'chun'],
  ['词', 'ci'], ['此', 'ci'], ['次', 'ci'],
  ['从', 'cong'], ['存', 'cun'], ['村', 'cun'],
  ['错', 'cuo'],
  ['打', 'da'], ['大', 'da'], ['达', 'da'], ['答', 'da'],
  ['带', 'dai'], ['待', 'dai'], ['代', 'dai'],
  ['单', 'dan'], ['但', 'dan'], ['蛋', 'dan'], ['淡', 'dan'],
  ['当', 'dang'], ['导', 'dao'], ['到', 'dao'], ['道', 'dao'], ['倒', 'dao'],
  ['得', 'de'], ['的', 'de'],
  ['灯', 'deng'], ['等', 'deng'],
  ['低', 'di'], ['底', 'di'], ['地', 'di'], ['弟', 'di'],
  ['点', 'dian'], ['电', 'dian'], ['店', 'dian'],
  ['调', 'diao'], ['掉', 'diao'],
  ['爹', 'die'],
  ['顶', 'ding'], ['定', 'ding'],
  ['丢', 'diu'],
  ['东', 'dong'], ['冬', 'dong'], ['动', 'dong'], ['懂', 'dong'],
  ['都', 'dou'], ['斗', 'dou'],
  ['读', 'du'], ['独', 'du'], ['度', 'du'],
  ['短', 'duan'], ['段', 'duan'], ['断', 'duan'],
  ['对', 'dui'], ['队', 'dui'],
  ['多', 'duo'],
  ['额', 'e'], ['恶', 'e'], ['饿', 'e'],
  ['儿', 'er'], ['耳', 'er'], ['二', 'er'],
  ['法', 'fa'], ['发', 'fa'], ['罚', 'fa'], ['伐', 'fa'], ['乏', 'fa'],
  ['翻', 'fan'], ['凡', 'fan'], ['烦', 'fan'], ['范', 'fan'],
  ['方', 'fang'], ['防', 'fang'], ['房', 'fang'], ['放', 'fang'],
  ['非', 'fei'], ['飞', 'fei'], ['费', 'fei'],
  ['分', 'fen'], ['份', 'fen'], ['纷', 'fen'], ['粉', 'fen'],
  ['风', 'feng'], ['封', 'feng'], ['峰', 'feng'], ['锋', 'feng'], ['凤', 'feng'],
  ['佛', 'fo'],
  ['该', 'gai'], ['改', 'gai'], ['盖', 'gai'], ['概', 'gai'],
  ['干', 'gan'], ['感', 'gan'],
  ['刚', 'gang'], ['钢', 'gang'],
  ['高', 'gao'], ['告', 'gao'],
  ['歌', 'ge'], ['个', 'ge'], ['给', 'gei'],
  ['跟', 'gen'], ['根', 'gen'],
  ['工', 'gong'], ['公', 'gong'], ['共', 'gong'], ['功', 'gong'], ['供', 'gong'],
  ['过', 'guo'], ['国', 'guo'], ['果', 'guo'],
  ['哈', 'ha'],
  ['还', 'hai'], ['海', 'hai'], ['害', 'hai'],
  ['韩', 'han'], ['汉', 'han'],
  ['号', 'hao'], ['好', 'hao'], ['浩', 'hao'],
  ['喝', 'he'], ['和', 'he'], ['合', 'he'], ['何', 'he'], ['河', 'he'],
  ['黑', 'hei'], ['很', 'hen'],
  ['红', 'hong'],
  ['后', 'hou'], ['厚', 'hou'],
  ['呼', 'hu'], ['户', 'hu'], ['互', 'hu'], ['护', 'hu'], ['虎', 'hu'], ['胡', 'hu'],
  ['花', 'hua'], ['化', 'hua'], ['划', 'hua'], ['华', 'hua'], ['话', 'hua'], ['画', 'hua'],
  ['换', 'huan'],
  ['黄', 'huang'],
  ['回', 'hui'], ['汇', 'hui'], ['灰', 'hui'], ['挥', 'hui'], ['辉', 'hui'], ['毁', 'hui'],
  ['会', 'hui'],
  ['婚', 'hun'], ['混', 'hun'],
  ['活', 'huo'], ['火', 'huo'], ['或', 'huo'], ['货', 'huo'],
  ['基', 'ji'], ['机', 'ji'], ['鸡', 'ji'], ['积', 'ji'], ['击', 'ji'], ['及', 'ji'], ['级', 'ji'], ['极', 'ji'], ['集', 'ji'], ['急', 'ji'], ['济', 'ji'],
  ['计', 'ji'], ['记', 'ji'], ['纪', 'ji'], ['继', 'ji'], ['技', 'ji'], ['季', 'ji'], ['际', 'ji'],
  ['家', 'jia'], ['加', 'jia'], ['价', 'jia'], ['架', 'jia'], ['假', 'jia'], ['嫁', 'jia'],
  ['间', 'jian'], ['件', 'jian'], ['建', 'jian'], ['渐', 'jian'],
  ['江', 'jiang'], ['将', 'jiang'], ['讲', 'jiang'], ['奖', 'jiang'], ['浆', 'jiang'], ['蒋', 'jiang'], ['匠', 'jiang'],
  ['角', 'jiao'], ['脚', 'jiao'], ['叫', 'jiao'], ['觉', 'jiao'], ['校', 'jiao'], ['较', 'jiao'],
  ['街', 'jie'], ['接', 'jie'], ['节', 'jie'], ['姐', 'jie'], ['解', 'jie'], ['介', 'jie'],
  ['今', 'jin'], ['金', 'jin'], ['仅', 'jin'], ['紧', 'jin'], ['进', 'jin'], ['近', 'jin'], ['晋', 'jin'],
  ['经', 'jing'], ['精', 'jing'], ['井', 'jing'], ['静', 'jing'], ['境', 'jing'], ['镜', 'jing'],
  ['纠', 'jiu'], ['九', 'jiu'], ['酒', 'jiu'], ['久', 'jiu'], ['旧', 'jiu'], ['就', 'jiu'],
  ['举', 'ju'], ['据', 'ju'], ['具', 'ju'], ['剧', 'ju'], ['聚', 'ju'], ['巨', 'ju'], ['句', 'ju'],
  ['卷', 'juan'],
  ['决', 'jue'], ['绝', 'jue'],
  ['卡', 'ka'],
  ['开', 'kai'],
  ['看', 'kan'],
  ['康', 'kang'],
  ['考', 'kao'], ['靠', 'kao'],
  ['科', 'ke'], ['可', 'ke'], ['课', 'ke'], ['客', 'ke'], ['刻', 'ke'], ['克', 'ke'],
  ['空', 'kong'], ['恐', 'kong'],
  ['口', 'kou'],
  ['哭', 'ku'], ['苦', 'ku'], ['库', 'ku'], ['裤', 'ku'],
  ['夸', 'kua'],
  ['快', 'kuai'], ['块', 'kuai'],
  ['宽', 'kuan'],
  ['况', 'kuang'], ['矿', 'kuang'],
  ['亏', 'kui'], ['愧', 'kui'],
  ['困', 'kun'],
  ['拉', 'la'], ['来', 'lai'], ['赖', 'lai'],
  ['蓝', 'lan'], ['兰', 'lan'], ['拦', 'lan'], ['栏', 'lan'], ['懒', 'lan'], ['烂', 'lan'],
  ['浪', 'lang'],
  ['老', 'lao'],
  ['乐', 'le'], ['勒', 'le'],
  ['雷', 'lei'], ['累', 'lei'], ['类', 'lei'], ['泪', 'lei'],
  ['冷', 'leng'],
  ['离', 'li'], ['里', 'li'], ['理', 'li'], ['礼', 'li'], ['李', 'li'], ['力', 'li'], ['历', 'li'], ['立', 'li'], ['利', 'li'], ['例', 'li'], ['隶', 'li'],
  ['连', 'lian'], ['联', 'lian'], ['廉', 'lian'], ['脸', 'lian'], ['练', 'lian'], ['恋', 'lian'],
  ['良', 'liang'], ['两', 'liang'], ['量', 'liang'], ['亮', 'liang'], ['凉', 'liang'], ['粮', 'liang'], ['梁', 'liang'],
  ['林', 'lin'], ['临', 'lin'], ['邻', 'lin'],
  ['领', 'ling'], ['另', 'ling'], ['令', 'ling'],
  ['留', 'liu'], ['流', 'liu'], ['刘', 'liu'], ['六', 'liu'],
  ['龙', 'long'],
  ['楼', 'lou'],
  ['陆', 'lu'], ['录', 'lu'], ['鹿', 'lu'], ['路', 'lu'],
  ['旅', 'lv'], ['绿', 'lv'], ['率', 'lv'], ['滤', 'lv'],
  ['乱', 'luan'],
  ['略', 'lve'],
  ['伦', 'lun'], ['轮', 'lun'], ['论', 'lun'],
  ['罗', 'luo'], ['洛', 'luo'], ['骆', 'luo'],
  ['马', 'ma'], ['妈', 'ma'], ['麻', 'ma'], ['码', 'ma'],
  ['买', 'mai'], ['卖', 'mai'], ['麦', 'mai'],
  ['满', 'man'], ['慢', 'man'],
  ['忙', 'mang'],
  ['毛', 'mao'], ['矛', 'mao'], ['冒', 'mao'], ['帽', 'mao'],
  ['么', 'me'],
  ['没', 'mei'], ['每', 'mei'], ['美', 'mei'], ['妹', 'mei'],
  ['门', 'men'], ['们', 'men'],
  ['梦', 'meng'],
  ['米', 'mi'],
  ['面', 'mian'],
  ['民', 'min'], ['敏', 'min'],
  ['名', 'ming'], ['明', 'ming'], ['命', 'ming'],
  ['摸', 'mo'], ['末', 'mo'], ['模', 'mo'], ['墨', 'mo'], ['莫', 'mo'],
  ['某', 'mou'],
  ['母', 'mu'], ['木', 'mu'], ['目', 'mu'], ['墓', 'mu'], ['幕', 'mu'],
  ['拿', 'na'], ['哪', 'na'], ['那', 'na'], ['纳', 'na'], ['娜', 'na'],
  ['男', 'nan'], ['南', 'nan'], ['难', 'nan'],
  ['呢', 'ne'],
  ['内', 'nei'],
  ['能', 'neng'],
  ['你', 'ni'],
  ['年', 'nian'], ['念', 'nian'],
  ['娘', 'niang'],
  ['鸟', 'niao'],
  ['您', 'nin'],
  ['牛', 'niu'],
  ['农', 'nong'], ['弄', 'nong'],
  ['怒', 'nu'],
  ['女', 'nv'],
  ['暖', 'nuan'],
  ['欧', 'ou'], ['偶', 'ou'],
  ['怕', 'pa'], ['拍', 'pai'], ['排', 'pai'], ['派', 'pai'],
  ['潘', 'pan'], ['盘', 'pan'], ['判', 'pan'], ['叛', 'pan'], ['盼', 'pan'],
  ['庞', 'pang'],
  ['跑', 'pao'],
  ['朋', 'peng'], ['配', 'pei'], ['碰', 'peng'],
  ['皮', 'pi'],
  ['偏', 'pian'], ['片', 'pian'],
  ['票', 'piao'], ['飘', 'piao'],
  ['频', 'pin'], ['品', 'pin'],
  ['平', 'ping'], ['评', 'ping'], ['凭', 'ping'], ['苹', 'ping'],
  ['破', 'po'],
  ['普', 'pu'], ['谱', 'pu'], ['朴', 'pu'],
  ['期', 'qi'], ['七', 'qi'], ['欺', 'qi'], ['其', 'qi'], ['奇', 'qi'], ['骑', 'qi'], ['起', 'qi'], ['启', 'qi'],
  ['气', 'qi'], ['汽', 'qi'], ['器', 'qi'], ['企', 'qi'], ['棋', 'qi'], ['旗', 'qi'], ['弃', 'qi'],
  ['恰', 'qia'], ['洽', 'qia'],
  ['千', 'qian'], ['前', 'qian'], ['钱', 'qian'], ['浅', 'qian'],
  ['强', 'qiang'], ['墙', 'qiang'], ['抢', 'qiang'],
  ['悄', 'qiao'], ['桥', 'qiao'], ['巧', 'qiao'],
  ['青', 'qing'], ['轻', 'qing'], ['倾', 'qing'], ['清', 'qing'], ['情', 'qing'], ['请', 'qing'], ['庆', 'qing'],
  ['穷', 'qiong'],
  ['秋', 'qiu'], ['球', 'qiu'], ['求', 'qiu'],
  ['区', 'qu'], ['曲', 'qu'], ['取', 'qu'], ['去', 'qu'], ['趣', 'qu'],
  ['全', 'quan'], ['权', 'quan'], ['泉', 'quan'],
  ['却', 'que'],
  ['群', 'qun'],
  ['然', 'ran'], ['让', 'rang'], ['绕', 'rao'],
  ['热', 're'],
  ['人', 'ren'], ['认', 'ren'],
  ['日', 'ri'],
  ['容', 'rong'], ['绒', 'rong'], ['荣', 'rong'], ['融', 'rong'],
  ['肉', 'rou'],
  ['如', 'ru'], ['入', 'ru'],
  ['软', 'ruan'],
  ['瑞', 'rui'],
  ['润', 'run'],
  ['若', 'ruo'],
  ['撒', 'sa'], ['洒', 'sa'], ['萨', 'sa'],
  ['三', 'san'], ['散', 'san'],
  ['嗓', 'sang'], ['丧', 'sang'],
  ['扫', 'sao'],
  ['色', 'se'], ['森', 'sen'], ['僧', 'seng'],
  ['沙', 'sha'], ['啥', 'sha'], ['杀', 'sha'],
  ['上', 'shang'],
  ['少', 'shao'], ['绍', 'sha'], ['哨', 'shao'], ['稍', 'shao'],
  ['社', 'she'], ['舍', 'she'], ['设', 'she'], ['射', 'she'], ['涉', 'she'],
  ['深', 'shen'], ['神', 'shen'], ['审', 'shen'], ['婶', 'shen'],
  ['生', 'sheng'], ['声', 'sheng'], ['升', 'sheng'], ['胜', 'sheng'], ['盛', 'sheng'], ['剩', 'sheng'],
  ['师', 'shi'], ['时', 'shi'], ['实', 'shi'], ['拾', 'shi'], ['食', 'shi'], ['使', 'shi'], ['始', 'shi'],
  ['世', 'shi'], ['市', 'shi'], ['示', 'shi'], ['式', 'shi'], ['事', 'shi'], ['是', 'shi'], ['室', 'shi'],
  ['试', 'shi'], ['视', 'shi'], ['释', 'shi'], ['适', 'shi'],
  ['收', 'shou'], ['守', 'shou'], ['首', 'shou'], ['受', 'shou'], ['授', 'shou'], ['售', 'shou'],
  ['书', 'shu'], ['术', 'shu'], ['树', 'shu'], ['竖', 'shu'], ['数', 'shu'], ['耍', 'shu'], ['摔', 'shu'],
  ['双', 'shuang'],
  ['水', 'shui'], ['睡', 'shui'],
  ['顺', 'shun'],
  ['说', 'shuo'], ['硕', 'shuo'], ['朔', 'shuo'],
  ['思', 'si'], ['私', 'si'], ['司', 'si'], ['丝', 'si'], ['死', 'si'], ['四', 'si'], ['似', 'si'],
  ['松', 'song'], ['送', 'song'], ['宋', 'song'], ['讼', 'song'],
  ['搜', 'sou'],
  ['素', 'su'], ['速', 'su'], ['诉', 'su'], ['塑', 'su'],
  ['酸', 'suan'], ['算', 'suan'],
  ['虽', 'sui'], ['岁', 'sui'], ['碎', 'sui'], ['遂', 'sui'],
  ['孙', 'sun'], ['损', 'sun'],
  ['所', 'suo'], ['索', 'suo'],
  ['他', 'ta'], ['她', 'ta'], ['它', 'ta'], ['塔', 'ta'],
  ['太', 'tai'], ['态', 'tai'], ['台', 'tai'], ['抬', 'tai'],
  ['谈', 'tan'], ['叹', 'tan'], ['碳', 'tan'], ['探', 'tan'],
  ['汤', 'tang'], ['糖', 'tang'], ['堂', 'tang'], ['唐', 'tang'],
  ['逃', 'tao'], ['桃', 'tao'], ['淘', 'tao'], ['讨', 'tao'],
  ['特', 'te'],
  ['疼', 'teng'],
  ['梯', 'ti'], ['提', 'ti'], ['题', 'ti'], ['体', 'ti'], ['替', 'ti'],
  ['天', 'tian'], ['田', 'tian'], ['甜', 'tian'], ['填', 'tian'],
  ['铁', 'tie'],
  ['听', 'ting'], ['停', 'ting'], ['庭', 'ting'],
  ['通', 'tong'], ['同', 'tong'], ['铜', 'tong'], ['童', 'tong'], ['统', 'tong'], ['痛', 'tong'],
  ['头', 'tou'], ['投', 'tou'], ['透', 'tou'],
  ['图', 'tu'], ['土', 'tu'], ['团', 'tuan'],
  ['推', 'tui'], ['腿', 'tui'], ['退', 'tui'],
  ['托', 'tuo'], ['拖', 'tuo'], ['脱', 'tuo'],
  ['挖', 'wa'], ['娃', 'wa'], ['瓦', 'wa'],
  ['外', 'wai'],
  ['湾', 'wan'], ['完', 'wan'], ['玩', 'wan'], ['晚', 'wan'], ['碗', 'wan'], ['万', 'wan'],
  ['王', 'wang'], ['往', 'wang'], ['网', 'wang'], ['忘', 'wang'], ['望', 'wang'], ['旺', 'wang'],
  ['危', 'wei'], ['微', 'wei'], ['为', 'wei'], ['未', 'wei'], ['位', 'wei'], ['卫', 'wei'],
  ['温', 'wen'], ['文', 'wen'], ['闻', 'wen'], ['稳', 'wen'], ['问', 'wen'],
  ['我', 'wo'], ['握', 'wo'], ['卧', 'wo'],
  ['污', 'wu'], ['屋', 'wu'], ['无', 'wu'], ['五', 'wu'], ['午', 'wu'], ['物', 'wu'], ['务', 'wu'],
  ['西', 'xi'], ['息', 'xi'], ['希', 'xi'], ['习', 'xi'], ['洗', 'xi'], ['喜', 'xi'], ['系', 'xi'],
  ['戏', 'xi'], ['细', 'xi'], ['隙', 'xi'],
  ['夏', 'xia'], ['吓', 'xia'], ['下', 'xia'],
  ['先', 'xian'], ['鲜', 'xian'], ['弦', 'xian'], ['险', 'xian'], ['县', 'xian'], ['线', 'xian'], ['限', 'xian'],
  ['想', 'xiang'], ['响', 'xiang'],
  ['小', 'xiao'], ['校', 'xiao'], ['笑', 'xiao'], ['效', 'xiao'],
  ['些', 'xie'], ['写', 'xie'], ['血', 'xie'],
  ['新', 'xin'], ['心', 'xin'], ['信', 'xin'],
  ['星', 'xing'], ['行', 'xing'], ['形', 'xing'], ['醒', 'xing'], ['姓', 'xing'], ['兴', 'xing'],
  ['凶', 'xiong'], ['胸', 'xiong'], ['雄', 'xiong'], ['熊', 'xiong'],
  ['修', 'xiu'], ['羞', 'xiu'], ['秀', 'xiu'], ['袖', 'xiu'],
  ['需', 'xu'], ['须', 'xu'], ['虚', 'xu'], ['徐', 'xu'], ['许', 'xu'], ['续', 'xu'],
  ['宣', 'xuan'], ['旋', 'xuan'], ['悬', 'xuan'], ['选', 'xuan'],
  ['学', 'xue'], ['雪', 'xue'],
  ['寻', 'xun'], ['巡', 'xun'], ['训', 'xun'], ['讯', 'xun'], ['迅', 'xun'],
  ['压', 'ya'], ['呀', 'ya'], ['鸭', 'ya'], ['牙', 'ya'], ['芽', 'ya'],
  ['演', 'yan'], ['延', 'yan'], ['言', 'yan'], ['研', 'yan'], ['眼', 'yan'],
  ['阳', 'yang'], ['养', 'yang'], ['样', 'yang'], ['洋', 'yang'], ['仰', 'yang'], ['氧', 'yang'],
  ['邀', 'yao'], ['腰', 'yao'], ['摇', 'yao'], ['窑', 'yao'], ['遥', 'yao'], ['咬', 'yao'],
  ['药', 'yao'], ['要', 'yao'], ['耀', 'yao'],
  ['爷', 'ye'], ['也', 'ye'], ['夜', 'ye'], ['业', 'ye'], ['叶', 'ye'], ['页', 'ye'], ['液', 'ye'],
  ['一', 'yi'], ['医', 'yi'], ['衣', 'yi'], ['依', 'yi'], ['仪', 'yi'], ['宜', 'yi'],
  ['移', 'yi'], ['遗', 'yi'], ['疑', 'yi'], ['已', 'yi'], ['以', 'yi'],
  ['艺', 'yi'], ['议', 'yi'], ['易', 'yi'], ['意', 'yi'], ['义', 'yi'], ['益', 'yi'],
  ['忆', 'yi'], ['亿', 'yi'], ['译', 'yi'], ['异', 'yi'], ['役', 'yi'],
  ['因', 'yin'], ['音', 'yin'], ['银', 'yin'], ['引', 'yin'], ['印', 'yin'],
  ['应', 'ying'], ['英', 'ying'], ['影', 'ying'], ['映', 'ying'], ['硬', 'ying'],
  ['用', 'yong'], ['勇', 'yong'], ['永', 'yong'], ['涌', 'yong'], ['泳', 'yong'],
  ['优', 'you'], ['由', 'you'], ['油', 'you'], ['游', 'you'], ['友', 'you'], ['有', 'you'], ['又', 'you'], ['右', 'you'], ['幼', 'you'],
  ['于', 'yu'], ['与', 'yu'], ['予', 'yu'], ['宇', 'yu'], ['雨', 'yu'], ['语', 'yu'],
  ['玉', 'yu'], ['育', 'yu'], ['域', 'yu'], ['遇', 'yu'], ['喻', 'yu'], ['寓', 'yu'], ['御', 'yu'],
  ['元', 'yuan'], ['园', 'yuan'], ['原', 'yuan'], ['圆', 'yuan'], ['源', 'yuan'], ['袁', 'yuan'], ['援', 'yuan'], ['远', 'yuan'], ['怨', 'yuan'], ['院', 'yuan'], ['愿', 'yuan'],
  ['约', 'yue'], ['月', 'yue'], ['跃', 'yue'], ['越', 'yue'], ['岳', 'yue'],
  ['云', 'yun'], ['运', 'yun'], ['蕴', 'yun'], ['员', 'yun'], ['匀', 'yun'], ['孕', 'yun'],
  ['在', 'zai'], ['再', 'zai'],
  ['咱', 'zan'], ['暂', 'zan'],
  ['脏', 'zang'], ['葬', 'zang'],
  ['早', 'zao'], ['枣', 'zao'], ['造', 'zao'], ['皂', 'zao'], ['灶', 'zao'],
  ['责', 'ze'], ['则', 'ze'], ['泽', 'ze'],
  ['贼', 'zei'],
  ['怎', 'zen'],
  ['增', 'zeng'], ['赠', 'zeng'],
  ['扎', 'zha'], ['炸', 'zha'], ['渣', 'zha'], ['闸', 'zha'], ['眨', 'zha'], ['诈', 'zha'], ['栅', 'zha'], ['榨', 'zha'],
  ['摘', 'zhai'], ['宅', 'zhai'], ['窄', 'zhai'], ['债', 'zhai'],
  ['盏', 'zhan'], ['站', 'zhan'], ['战', 'zhan'], ['占', 'zhan'], ['展', 'zhan'],
  ['张', 'zhang'], ['掌', 'zhang'], ['涨', 'zhang'], ['账', 'zhang'], ['胀', 'zhang'], ['障', 'zhang'],
  ['招', 'zhao'], ['找', 'zhao'], ['照', 'zhao'], ['罩', 'zhao'], ['兆', 'zhao'], ['赵', 'zhao'],
  ['着', 'zhao'],
  ['者', 'zhe'], ['这', 'zhe'],
  ['折', 'zhe'], ['哲', 'zhe'],
  ['真', 'zhen'], ['针', 'zhen'], ['诊', 'zhen'], ['镇', 'zhen'], ['阵', 'zhen'], ['震', 'zhen'],
  ['争', 'zheng'], ['征', 'zheng'], ['正', 'zheng'], ['证', 'zheng'], ['政', 'zheng'], ['郑', 'zheng'],
  ['之', 'zhi'], ['支', 'zhi'], ['只', 'zhi'], ['织', 'zhi'], ['知', 'zhi'], ['指', 'zhi'],
  ['至', 'zhi'], ['治', 'zhi'], ['质', 'zhi'], ['致', 'zhi'], ['志', 'zhi'], ['置', 'zhi'],
  ['中', 'zhong'], ['钟', 'zhong'], ['终', 'zhong'], ['种', 'zhong'], ['重', 'zhong'],
  ['周', 'zhou'], ['州', 'zhou'], ['洲', 'zhou'], ['粥', 'zhou'], ['轴', 'zhou'], ['肘', 'zhou'], ['帚', 'zhou'], ['咒', 'zhou'], ['宙', 'zhou'], ['昼', 'zhou'], ['皱', 'zhou'], ['骤', 'zhou'],
  ['朱', 'zhu'], ['主', 'zhu'], ['注', 'zhu'], ['住', 'zhu'], ['助', 'zhu'],
  ['柱', 'zhu'], ['著', 'zhu'], ['铸', 'zhu'], ['筑', 'zhu'],
  ['抓', 'zhua'], ['爪', 'zhua'],
  ['拽', 'zhuai'],
  ['专', 'zhuan'], ['转', 'zhuan'], ['赚', 'zhuan'], ['撰', 'zhuan'],
  ['桩', 'zhuang'], ['装', 'zhuang'], ['庄', 'zhuang'], ['撞', 'zhuang'], ['壮', 'zhuang'], ['状', 'zhuang'],
  ['准', 'zhun'],
  ['桌', 'zhuo'], ['卓', 'zhuo'], ['琢', 'zhuo'], ['灼', 'zhuo'],
  ['苗', 'miao'], ['秒', 'miao'], ['妙', 'miao'], ['庙', 'miao'], ['描', 'miao'], ['瞄', 'miao'],
  ['灭', 'mie'],
  ['鸣', 'ming'], ['谬', 'miu'],
  ['摹', 'mo'], ['膜', 'mo'], ['磨', 'mo'], ['魔', 'mo'], ['沫', 'mo'], ['漠', 'mo'], ['默', 'mo'],
  ['牟', 'mou'],
  ['牡', 'mu'], ['亩', 'mu'], ['姆', 'mu'],
  ['暮', 'mu'], ['穆', 'mu'],
  ['拿', 'na'], ['呐', 'na'], ['钠', 'na'],
  ['晕', 'yun'],
  // 常用工具相关词组
  ['工具', 'gongju'], ['编码', 'bianma'], ['解码', 'jiema'], ['加密', 'jiami'], ['转换', 'zhuanhuan'],
  ['生成', 'shengcheng'], ['计算', 'jisuan'], ['验证', 'yanzheng'], ['格式化', 'geshihua'],
  ['对比', 'duibi'], ['压缩', 'yasuo'], ['解压', 'jieya'], ['图片', 'tupian'],
  ['文件', 'wenjian'], ['文本', 'wenben'], ['数据', 'shuju'], ['颜色', 'yanse'],
  ['时间', 'shijian'], ['日期', 'riqi'], ['坐标', 'zuobiao'], ['距离', 'juli'],
  ['地图', 'ditu'], ['网络', 'wangluo'], ['请求', 'qingqiu'], ['开发', 'kaifa'],
  ['日志', 'rizhi'], ['分析', 'fenxi'], ['办公', 'bangong'], ['处理', 'chuli'],
  ['管理', 'guanli'], ['查询', 'chaxun'], ['搜索', 'sousuo'], ['排序', 'paixu'],
  ['过滤', 'guolv'], ['归类', 'guilei'],
  ['经度', 'jingdu'], ['纬度', 'weidu'],
  ['距离', 'juli'], ['计算', 'jisuan'], ['转换', 'zhuanhuan'],
  ['显示', 'xianshi'], ['点', 'dian'], ['线', 'xian'], ['面', 'mian'], ['区域', 'quyu'],
  ['解码', 'jiema'], ['密', 'mi'], ['串', 'chuan'], ['格式', 'geshi'],
  ['清理', 'qingli'], ['优化', 'youhua'], ['删除', 'shanchu'], ['添加', 'tianjia'],
  ['修改', 'xiugai'], ['查看', 'chakan'], ['浏览', 'liulan'], ['列表', 'liebiao'],
  ['网址', 'wangzhi'], ['链接', 'lianjie'],
  ['系统', 'xitong'], ['内存', 'neicun'], ['磁盘', 'cipan'],
  ['注册', 'zhuce'], ['表', 'biao'], ['卸载', 'xiazai'], ['运行', 'yunxing'],
  ['速度', 'sudu'], ['网卡', 'wangqia'], ['声卡', 'shengqia'],
  ['安全', 'anquan'], ['隐私', 'yinsi'], ['保护', 'baohu'], ['密码', 'mima'],
  ['密钥', 'miyue'], ['令牌', 'lingpai'], ['认证', 'renzheng'], ['授权', 'shouquan'], ['权限', 'quanxian'],
  ['调试', 'tiaoshi'], ['源码', 'yuama'], ['编译', 'bianyi'], ['环境', 'huanjing'],
  ['包', 'bao'], ['依赖', 'yilai'],
  ['获取', 'huoqu'], ['设置', 'shezhi'], ['输入', 'shuru'], ['输出', 'shuchu'],
  ['格式', 'geshi'], ['更新', 'gengxin'], ['复制', 'fuzhi'], ['粘贴', 'zhantie'],
  ['撤销', 'chexiao'], ['取消', 'quxiao'],
  ['名称', 'mingcheng'], ['类型', 'leixing'], ['路径', 'lujing'],
  ['大小', 'daxiao'], ['尺寸', 'chicun'], ['宽度', 'kuandu'], ['高度', 'gaodu'],
  ['长度', 'changdu'], ['面积', 'mianji'], ['体积', 'tiji'], ['容量', 'rongliang'],
  ['图标', 'tubiao'], ['导航', 'daohang'], ['搜索', 'sousuo'], ['入口', 'rukou'],
  ['框', 'kuang'], ['提示', 'tishi'], ['历史', 'lishi'], ['记录', 'jilu'],
  ['标题', 'biaoti'], ['栏目', 'lanmu'], ['菜单', 'caidan'], ['工具', 'gongju'],
  ['状态', 'zhuangtai'], ['内容', 'neirong'], ['广告', 'guanggao'],
  ['弹窗', 'danchuang'], ['对话框', 'duihuakuang'], ['警告', 'jinggao'],
  ['正确', 'zhengque'], ['失败', 'shibai'], ['成功', 'chenggong'],
  ['完成', 'wancheng'], ['中等', 'zhongdeng'], ['等待', 'dengdai'],
  ['性能', 'xingneng'], ['占用', 'zhanyong'], ['释放', 'shifang'],
  ['回收', 'huishou'], ['机制', 'jizhi'], ['垃圾', 'laji'],
  ['临时', 'linshi'], ['cookies', 'cookies'],
  ['表单', 'biaodan'], ['自动', 'zidong'], ['填写', 'tianxie'],
  ['管理器', 'guanliqi'], ['网页', 'wangye'], ['保存', 'baocun'],
  ['快照', 'kuangzhao'], ['打印', 'dayin'], ['设置', 'shezhi'],
  ['打印机', 'dayinji'], ['参数', 'canshu'], ['纸张', 'zhizhang'],
  ['方向', 'fangxiang'], ['纵向', 'zongxiang'], ['横向', 'hengxiang'],
  ['页眉', 'yuemei'], ['脚注', 'jiaozhu'], ['边距', 'bianju'],
  ['文字', 'wenzi'], ['字体', 'ziti'], ['字号', 'zihao'],
  ['缩放', 'suofang'], ['比例', 'bili'],
  ['图像', 'tuxiang'], ['分辨率', 'fenbianlv'],
  ['裁剪', 'caijian'], ['旋转', 'xuanzhuan'], ['翻转', 'fanzhuan'],
  ['修色', 'xiuse'], ['色调', 'sediao'],
  ['亮度', 'liangdu'], ['对比度', 'duibidu'],
  ['饱和度', 'baohedu'],
  ['音频', 'yinpin'], ['频率', 'pinlu'], ['剪辑', 'jianji'],
  ['视频', 'shipin'], ['帧率', 'zhenglu'],
  ['字符', 'zifu'], ['字符集', 'zifuji'], ['正则', 'zhengze'],
  ['表达式', 'biaodashi'], ['二维码', 'erweima'], ['条码', 'tiaooma'],
  ['标签', 'biaoqian'], ['票据', 'piaoju'], ['发票', 'fapiao'],
  ['收据', 'shouju'], ['数据库', 'shujuku'], ['备份', 'beifen'], ['恢复', 'huifu'],
  ['导出', 'daochu'], ['导入', 'daoru'],
  ['哈希', 'hash'], ['校验', 'jiaoyan'], ['uuid', 'uuid'],
  ['时间戳', 'shijianchuo'], ['时区', 'shiqu'],
  ['crud', 'crud'], ['csv', 'csv'], ['json', 'json'], ['yaml', 'yaml'],
  ['xml', 'xml'], ['url', 'url'],
  ['进制', 'jinzhi'], ['base', 'base'],
  ['正则', 'zhengze'], ['匹配', 'pipei'],
  ['ip', 'ip'], ['端口', 'duankou'], ['域名', 'yumimg'],
  ['颜色', 'yanse'], ['配色', 'peise'],
  ['快捷键', 'kuaijiejian'], ['手势', 'shoushi'],
  ['消息', 'xiaoxi'], ['推送', 'tuisong'], ['接收', 'jieshou'],
  ['特效', 'texiao'], ['动画', 'donghua'], ['表格', 'biaoge'],
  ['日程', 'richeng'], ['计时', 'jishi'], ['闹钟', 'naozhong'],
  ['周期', 'zhouqi'], ['任务', 'renwu'], ['床', 'chuang'],
  ['箱', 'xiang'], ['命令行', 'minglinghang'], ['脚本', 'jiaoben'],
  ['屏幕', 'pingmu'], ['截图', 'jietu'], ['录制', 'luzhi'],
  ['键盘', 'jianpan'], ['鼠标', 'shubiao'], ['连点', 'liandian'],
  ['语音', 'yuyin'], ['识别', 'shibie'], ['合成', 'hecheng'], ['输入', 'shuru'],
  ['输入法', 'shurufa'],
  ['进程', 'jincheng'], ['内存', 'neicun'], ['碎片', 'suipian'],
  ['在线', 'zaixian'], ['离线', 'lixian'], ['本地', 'bendi'],
  ['软件', 'ruanjian'], ['系统', 'xitong'], ['平台', 'pingtai'],
  ['门户', 'menhu'], ['网站', 'wangzhan'], ['社区', 'shequ'],
  ['论坛', 'luntan'], ['博客', 'boke'], ['商城', 'shangcheng'],
  ['店铺', 'dianpu'], ['企业', 'qiye'], ['商家', 'shangjia'],
  ['个人', 'geren'], ['微型', 'weixing'], ['小型', 'xiaoxing'],
  ['大型', 'daxing'],
  ['面板', 'mianban'], ['组件', 'zujian'], ['列表', 'liebiao'],
  ['网格', 'wangge'], ['树形', 'shuxing'], ['画廊', 'hualang'],
  ['灯箱', 'dengxiang'], ['缩略', 'suolue'], ['散点', 'sandian'],
  ['雷达', 'leida'], ['热力', 'reli'], ['思维', 'siwei'],
  ['导图', 'daotu'], ['生产', 'shengchan'], ['流程', 'liucheng'],
];

// 转换为 Map 以提高查询效率
const pinyinMap = new Map<string, string>(pinyinData);

// 声母表 (保留用于参考)
// const _initials = ['zh', 'ch', 'sh', 'b', 'p', 'm', 'f', 'd', 't', 'n', 'l', 'g', 'k', 'h', 'j', 'q', 'x', 'r', 'z', 'c', 's', 'y', 'w'];

// 全拼缓存
const pinyinCache = new Map<string, string>();

/**
 * 判断字符是否为汉字
 */
export function isChinese(char: string): boolean {
  return /[\u4e00-\u9fa5]/.test(char);
}

/**
 * 获取单个汉字的拼音
 */
export function getPinyin(char: string): string {
  return pinyinMap.get(char) || '';
}

/**
 * 获取汉字字符串的拼音
 */
export function getPinyinString(str: string): string {
  if (pinyinCache.has(str)) {
    return pinyinCache.get(str)!;
  }

  let result = '';
  for (const char of str) {
    if (isChinese(char)) {
      result += getPinyin(char);
    } else {
      result += char.toLowerCase();
    }
  }

  pinyinCache.set(str, result);
  return result;
}

/**
 * 获取拼音首字母
 */
export function getPinyinInitials(str: string): string {
  const pinyin = getPinyinString(str);
  let result = '';
  let i = 0;

  while (i < pinyin.length) {
    // 检查两字符声母 (zh, ch, sh)
    if (i + 1 < pinyin.length) {
      const twoChars = pinyin.substring(i, i + 2);
      if (twoChars === 'zh' || twoChars === 'ch' || twoChars === 'sh') {
        result += twoChars[0];
        i += 2;
        continue;
      }
    }

    // 单字符
    result += pinyin[i];
    i++;
  }

  return result;
}

/**
 * 检查查询字符串是否匹配目标字符串（模糊匹配）
 * 支持：直接匹配、拼音匹配、首字母匹配、拼音模糊匹配
 */
export function fuzzyMatch(query: string, target: string, targetPinyin: string, targetInitials: string): boolean {
  const q = query.toLowerCase().trim();
  if (!q) return true;

  // 精确包含
  if (target.toLowerCase().includes(q)) return true;

  // 拼音全拼匹配
  if (targetPinyin.includes(q)) return true;

  // 首字母匹配
  if (targetInitials.toLowerCase().includes(q)) return true;

  // 拼音模糊匹配（查询词的每个字符对应拼音的连续部分）
  if (fuzzyPinyinMatch(q, targetPinyin)) return true;

  return false;
}

/**
 * 拼音模糊匹配
 * 例如查询 "zd" 可以匹配 "zhongwen" (zhong + d)
 */
function fuzzyPinyinMatch(query: string, pinyin: string): boolean {
  let queryIndex = 0;
  let pinyinIndex = 0;

  while (queryIndex < query.length && pinyinIndex < pinyin.length) {
    if (query[queryIndex] === pinyin[pinyinIndex]) {
      queryIndex++;
      pinyinIndex++;
    } else {
      pinyinIndex++;
    }
  }

  return queryIndex === query.length;
}

/**
 * 搜索工具
 * 支持：模糊搜索、拼音搜索、首字母搜索、拼音模糊搜索
 */
export function searchTools<T extends { name: string; description: string; tags: string[] }>(
  tools: T[],
  query: string
): T[] {
  const q = query.toLowerCase().trim();
  if (!q) return tools;

  return tools.filter(tool => {
    // 获取拼音信息
    const namePinyin = getPinyinString(tool.name.toLowerCase());
    const nameInitials = getPinyinInitials(tool.name.toLowerCase());

    // 名称匹配
    if (fuzzyMatch(q, tool.name, namePinyin, nameInitials)) return true;

    // 描述匹配
    const descPinyin = getPinyinString(tool.description.toLowerCase());
    const descInitials = getPinyinInitials(tool.description.toLowerCase());
    if (fuzzyMatch(q, tool.description, descPinyin, descInitials)) return true;

    // 标签匹配
    for (const tag of tool.tags) {
      const tagPinyin = getPinyinString(tag.toLowerCase());
      const tagInitials = getPinyinInitials(tag.toLowerCase());
      if (fuzzyMatch(q, tag, tagPinyin, tagInitials)) return true;
    }

    return false;
  });
}
