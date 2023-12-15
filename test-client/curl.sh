# for prompt in a list of string

prompts=(
    "周杰伦,鸡腿,汉堡"
    "皮卡丘,芒果,蛋挞"
    "周杰伦,奶油,薯条"
    "酥软的,龙虾,薯条"
    "柔嫩的,布丁,鸡米花"
    "鲜花,芒果,蛋挞"
    "酥软的,牛蛙,薯条"
    "烟雾,葱花,汉堡"
    "酥软的,龙虾,汉堡"
    "天空,泡菜,鸡米花"
    "极品的,巧克力,薯条"
    "麻辣的,牛蛙,汉堡"
    "滑嫩的,苹果,薯条"
    "宇宙,芝麻,薯条"
    "水光,椰子,鸡米花"
    "春光,蒜头,蛋挞"
    "怀旧的,薯片,鸡米花"
    "细腻的,猪肉,蛋挞"
    "口感鲜美的,桃子,汉堡"
    "酸甜的,番茄,薯条"
    "甜美的,奶酪,汉堡"
    "星空,蛋挞,鸡米花"
    "珊瑚,咸蛋,原味鸡"
    "口感细腻的,鹅肉,薯条"
    "黎明,紫薯,原味鸡"
    "咸鲜的,香肠,蛋挞"
    "静谧的,草莓,原味鸡"
    "绿洲,石榴,蛋挞"
    "城堡,菠萝,蛋挞"
    "瀑布,香菜,汉堡"
    "精致的,大米,蛋挞"
    "醇香的,萝卜,鸡米花"
    "脆口的,山药,汉堡"
    "音乐,甜椒,原味鸡"
    "麻辣的,苦瓜,鸡米花"
    "诱人的,鸡肉,蛋挞"
    "彩虹,面包,鸡米花"
    "温暖的,芹菜,原味鸡"
    "绚烂的,猪蹄,薯条"
    "冰山,糯米,蛋挞"
    "未来的,黄豆,蛋挞"
    "奇幻的,红豆,蛋挞"
    "剪纸,蒜泥,汉堡"
    "朦胧的,米饭,原味鸡"
    "新鲜的,牛奶,汉堡"
    "清晨,鸡胸肉,原味鸡"
    "垂柳,豌豆尖,鸡米花"
    "醇厚的,南瓜,原味鸡"
    "朝霞,银耳,鸡米花"
    "丛林,西兰花,薯条"
    "浓郁的,咖啡,薯条"
    "云朵,火腿,汉堡"
    "曙光,莴苣,蛋挞"
    "辣味十足的,玉米,原味鸡"
    "树影,猕猴桃,鸡米花"
    "美味的,芝士,汉堡"
    "水滴,百合,薯条"
    "沙滩,柿子,蛋挞"
    "沙漠,黑米,汉堡"
    "花海,柠檬,汉堡"
    "梦幻的,杏鲍菇,蛋挞"
    "弹牙的,酱油,蛋挞"
    "绝佳的,鸭肉,薯条"
    "雪花,金枪鱼,薯条"
    "青葱,牛排,鸡米花"
    "枫叶,鱿鱼,原味鸡"
    "高山,蚕豆,薯条"
    "香咸的,杨梅,原味鸡"
    "滋补的,橙子,蛋挞"
    "绿野仙踪,鸡腿,汉堡"
    "柔软的,生菜,原味鸡"
    "清淡的,麦片,蛋挞"
    "光,黑木耳,汉堡"
    "清新的,葡萄,鸡米花"
    "无比美味的,鱼肉,薯条"
    "阳光,杨桃,薯条"
    "晨曦,杏仁,原味鸡"
    "雪山,鱼籽,薯条"
    "浪漫的,花生,鸡米花"
    "青山,凤爪,鸡米花"
    "油画,卷心菜,原味鸡"
    "香辣的,土豆,薯条"
    "香酥的,燕麦,鸡米花"
    "香甜的,鸡蛋,汉堡"
    "暮色,鲍鱼,薯条"
    "晶莹的,黄瓜,鸡米花"
    "寒意,龙眼,蛋挞"
    "湖泊,虾仁,蛋挞"
    "建筑,牛肉,鸡米花"
    "舞蹈,白酒,原味鸡"
    "口述的,葡萄柚,汉堡"
    "水晶,豌豆,汉堡"
    "水嫩的,樱桃,原味鸡"
    "鲜美的,豆腐,原味鸡"
    "温泉,蛋糕,汉堡"
    "黄昏,绿豆芽,原味鸡"
    "环保的,生姜,原味鸡"
    "夏夜,碧根果,薯条"
    "营养丰富的,洋葱,鸡米花"
    "果园,茶叶,汉堡"
    "美好的,香蕉,鸡米花"
    "雾霭,橄榄油,蛋挞"
    "刺激味蕾的,茼蒿,汉堡"
    "融化在口中的,牛油果,汉堡"
    "火山,豆腐干,薯条"
    "酥脆的,酸奶,汉堡"
    "落日,奶油,原味鸡"
)

for prompt in "${prompts[@]}"
do
    echo "$prompt"
    curl --location 'http://localhost:3000/api/yum/generate/image' \
    --header 'Content-Type: application/json' \
    --data '{
        "params": {
            "prompt": "'"$prompt"'",
            "negativePrompt": ""
        },
        "resultCallbackUrl": "http://localhost:3001/callback"
    }'
    echo "\n"
    sleep 1
done
