use serde_json::Value;

fn main() {
    let json_str = r#"{"filters":{"terms":{"city_code":330100,"toliet_exist":1},"range":{"shortest_subway_distance":[1,1000],"sort_price":[null,2100]},"geo_bounding_box":{"top_left":{"lat":30.295484586645692,"lon":120.11303158402404},"bottom_right":{"lat":30.26965402257079,"lon":120.1272068449715}}},"bool":{"must2":[{"should":[{"terms":{"product_category":"1"}}]}]},"groups":{"resblocks":{"fetch":{"return":["resblock_name","resblock_id","apartment_type","price_show","sort_price","grid_code","geohash7","district_code","district_name","loc"],"size":1,"sort":{"sort_price":"asc"}},"field":{"resblock_id":"terms"},"size":100}},"sorts":{"score":"desc"},"start":1,"returns":null,"uid":"81f44290-40f0-4cff-ac8f-4514dc3e16a7","imei":"96b36cce06598cc6","product":"1","query_session_id":"2009523079624949760","origin":"app","from_scene":"AppListMapResblock","scrollId":"","divideInfo":{"switch":0,"page":0,"offset":0},"mode":"room","params_plus":{"channel":"homepage_search_map_map","isFilter":0,"searchType":3,"os":"android","isMap":2,"filterType":"租房","filterLocation":"{}","filterRentType":"{\"feature_house\":\"近地铁\",\"layout\":\"主卧独卫\",\"type\":\"合租\"}","filterRentalPrice":"{\"price\":\"0,2100\"}","filterOther":"{\"feature_house\":\"近地铁\"}","sort":"","model":"23117RK66C","ip":"172.18.193.44"},"ab":{"companyListAB":"geohash7","zrasearchstyle":"doublestyle"},"page_source":"homepage_search_map_map","search_area":"","debug":false,"is_use_his_filter":true,"comprehensive_strategy":"sys","result_type":""}"#;

    match serde_json::from_str::<Value>(json_str) {
        Ok(value) => {
            println!("JSON 解析成功");
            match serde_json::to_string_pretty(&value) {
                Ok(formatted) => {
                    println!("JSON 格式化成功");
                    println!("长度: {}", formatted.len());
                }
                Err(e) => {
                    println!("JSON 格式化失败: {}", e);
                }
            }
        }
        Err(e) => {
            println!("JSON 解析失败: {}", e);
        }
    }
}
