use finata::utils::Client;
use reqwest::header::*;
use finata::Config;

fn client() -> Client {
    let mut client = Client::with_header(finata::hdmap! {
        USER_AGENT => finata::utils::UA.clone(),
        REFERER => "https://www.bilibili.com",
    });
    *client.client_mut() = reqwest::Client::new();
    client
}

#[cfg(test)]
mod bilibili {
    use finata::website::bilibili::*;
    use finata::Extract;
    use super::*;

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
}

#[cfg(test)]
mod netease_music {
    use finata::website::netease_music::*;
    use finata::Extract;
    use super::*;

    #[tokio::test]
    async fn song() {
        let mut extractor = Song::new("https://music.163.com/#/song?id=1440302397").unwrap();
        *extractor.client_mut() = client();
        let res = extractor.extract().await.unwrap();
        assert_eq!(res.title(), "Same Side");
        assert!(!res.raws().is_empty())
    }
    #[tokio::test]
    async fn playlist() {
        let mut extractor = PlayList::new("https://music.163.com/#/playlist?id=78674080").unwrap();
        *extractor.client_mut() = client();
        let res = extractor.extract().await.unwrap();
        assert_eq!(res.title(), "【古典音乐】治疗焦虑症的良药");
        assert!(!res.raws().is_empty())
    }
}
