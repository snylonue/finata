#[cfg(test)]
mod bilibili {
    use finata::utils::Client;
    use finata::website::bilibili::*;
    use finata::Config;
    use finata::Extract;
    use reqwest::header::*;

    fn client() -> Client {
        let mut client = Client::with_header(finata::hdmap! {
            USER_AGENT => finata::utils::UA.clone(),
            REFERER => "https://www.bilibili.com",
        });
        *client.client_mut() = reqwest::Client::new();
        client
    }
    #[tokio::test]
    async fn base_extractor() {
        let mut extractor = BaseExtractor::new(54592589, 95677892, client());
        let res = extractor.extract().await.unwrap();
        assert_eq!(
            res.title(),
            "第一部网络视频长啥样？让你大开眼界的中国网络视频发展史#01"
        );
        assert!(!res.raws().is_empty());
    }
    #[tokio::test]
    async fn av() {
        let mut extractor = Video::new("https://www.bilibili.com/video/av54592589/").unwrap();
        *extractor.client_mut() = client();
        let res = extractor.extract().await.unwrap();
        assert_eq!(
            res.title(),
            "第一部网络视频长啥样？让你大开眼界的中国网络视频发展史#01"
        );
        assert!(!res.raws().is_empty());
    }
    #[tokio::test]
    async fn bv() {
        let mut extractor = Video::new("https://www.bilibili.com/video/BV1L4411M7sC").unwrap();
        *extractor.client_mut() = client();
        let res = extractor.extract().await.unwrap();
        assert_eq!(
            res.title(),
            "第一部网络视频长啥样？让你大开眼界的中国网络视频发展史#01"
        );
        assert!(!res.raws().is_empty());
    }
    #[tokio::test]
    async fn ep() {
        let mut extractor = Bangumi::new("https://www.bilibili.com/bangumi/play/ep28251").unwrap();
        *extractor.client_mut() = client();
        let res = extractor.extract().await.unwrap();
        assert_eq!(res.title(), "【1月】路人女主的养成方法 03");
        assert!(!res.raws().is_empty());
    }
    #[tokio::test]
    async fn ss() {
        let mut extractor = Bangumi::new("https://www.bilibili.com/bangumi/play/ss1512").unwrap();
        *extractor.client_mut() = client();
        let res = extractor.extract().await.unwrap();
        assert_eq!(res.title(), "【1月】路人女主的养成方法 00");
        assert!(!res.raws().is_empty());
    }
    #[tokio::test]
    async fn fix_old() {
        let mut extractor = Bangumi::new("https://www.bilibili.com/bangumi/play/ss5051").unwrap();
        *extractor.client_mut() = client();
        let res = extractor.extract().await.unwrap();
        assert_eq!(res.title(), "【剧场版】planetarian ～星之人～【独家正版】");
        assert!(!res.raws().is_empty());
    }
}

#[cfg(test)]
mod netease_music {
    use finata::website::netease_music::*;
    use finata::Extract;

    #[tokio::test]
    async fn song() {
        let mut extractor = Song::new("https://music.163.com/#/song?id=1458308282").unwrap();
        let res = extractor.extract().await.unwrap();
        assert_eq!(
            res.title(),
            "嘘月（「想哭的我戴上猫的面具」片尾曲）（翻自 ヨルシカ）"
        );
        assert!(!res.raws().is_empty())
    }
}
