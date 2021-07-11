#[cfg(test)]
mod bilibili {
    use finata::website::bilibili::*;
    use finata::Extract;

    #[tokio::test]
    async fn av() {
        let mut extractor = Video::new("https://www.bilibili.com/video/av54592589").unwrap();
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
        let res = extractor.extract().await.unwrap();
        assert_eq!(res.title(), "【1月】路人女主的养成方法 03");
        assert!(!res.raws().is_empty());
    }
    #[tokio::test]
    async fn ss() {
        let mut extractor = Bangumi::new("https://www.bilibili.com/bangumi/play/ss1512").unwrap();
        let res = extractor.extract().await.unwrap();
        assert_eq!(res.title(), "【1月】路人女主的养成方法 00");
        assert!(!res.raws().is_empty());
    }
}
