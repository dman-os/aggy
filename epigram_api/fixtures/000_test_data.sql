-- strings use single quotes
BEGIN;

DO $body$
    DECLARE
        -- use variables in order to be able to access properties using the dot operator
    BEGIN
        INSERT INTO grams.grams (
            id
            ,created_at
            ,content
            ,coty
            ,parent_id
            ,sig
            ,author_pubkey
            ,author_alias
            ,author_notif_email
        ) 
        VALUES 
        (
            '\xc6d9d817d53dee6c0ae00205e9f32f6373b23215ddd442a5dce193cce73f5925'::bytea
            ,to_timestamp(1691479928)
            ,$$I wan't you to know, I wan't you to know that I'm awake.$$
            ,'text/html'
            ,NULL
            ,'\x06a6016f64de7f22123816cc6a00db5c3d7d62da64fcb42daba234e2f6ecbc4ea6bb1671d035c3ffdbe6ed2a92dafbd5341f1d107557043b8d2fe018f17fbe0e'::bytea
            ,'\x4ea301616a42cfbbd03f33570038156065fc217a86cdcb993e9fb9b197d08b53'::bytea
            ,'use1'
            ,'use1@aggy.news'
        )
        ,(
            '\x35a356563678440efa1eb44e5cb2036e5e31b9eb6f04ef5df0c70966d5226b12'::bytea
            ,to_timestamp(1691479928)
            ,$$And I hope you're asleep.$$
            ,'text/html'
            ,'\xc6d9d817d53dee6c0ae00205e9f32f6373b23215ddd442a5dce193cce73f5925'::bytea
            ,'\x519096262a6b214837dae999e8688d265bbed056207bc47fcf30e8a4b526b2bcd0e708f002f7c5d3ead38453a53a40735fb35fc56030902eb9a6eef03df66405'::bytea
            ,'\xe90bb6e011ed9b2607b45c6917405f56b5c793168c578343e353cde94c4b6bed'::bytea
            ,'fideroth'
            ,'fideroth@aggy.news'
        )
        ,(
            '\x6affc96805b62f5a4c47b1ef2cf436910eb4df0253c7226d94406e6ab2771de5'::bytea
            ,to_timestamp(1691479928)
            ,$$*air guitars madly*$$
            ,'text/html'
            ,'\x35a356563678440efa1eb44e5cb2036e5e31b9eb6f04ef5df0c70966d5226b12'::bytea
            ,'\x9805011ae871eadbf5ab8e8501c2697731361ce11410d8afa9af696f89ce059f27dbce9bee77dc41e9fa4c44a7adfa02250e4f09911c7bd45302f846ebbeac0e'::bytea
            ,'\x4ea301616a42cfbbd03f33570038156065fc217a86cdcb993e9fb9b197d08b53'::bytea
            ,'use1'
            ,'use1@aggy.news'
        )
        ,(
            '\x64eb58f1ee950ea7519039eb39690bd56c94065301246cb5d572b703bdaa6421'::bytea
            ,to_timestamp(1691479928)
            ,$$*sads doggly*$$
            ,'text/html'
            ,'\x6affc96805b62f5a4c47b1ef2cf436910eb4df0253c7226d94406e6ab2771de5'::bytea
            ,'\x8bc68f72d274ad8919a01a62e2b512175fec2be38211de1c760dcd775539f45da0509d725ee4171a8ff4d78a370ae179f857a3ff3c78da0f6cfb6bd9d076990b'::bytea
            ,'\xe90bb6e011ed9b2607b45c6917405f56b5c793168c578343e353cde94c4b6bed'::bytea
            ,'fideroth'
            ,'fideroth@aggy.news'
        )
        ,(
            '\x7ac84dba79c3c8c085e49f96a03a91ab24ad6c436b9c798d76e5bdf1a3b7de3d'::bytea
            ,to_timestamp(1691479928)
            ,$$What gives?$$
            ,'text/html'
            ,'\xc6d9d817d53dee6c0ae00205e9f32f6373b23215ddd442a5dce193cce73f5925'::bytea
            ,'\x32806c93b2a041d60cb2c66ae05bf70d8facbcda7175e1ed815cf61cfdffffde473c12188e6fc20f3d7943f427e19a2e34d76bd388317adfc9f007c83704690d'::bytea
            ,'\xd560a19636bf6ae6458c91b01d0658c382d629cbc5c04d3c028cc1d075e982b2'::bytea
            ,'the_i18n_man'
            ,'the_i18n_man@aggy.news'
        )
        ,(
            '\xba335c36a5f3a7e913f60dff09e296ae085fd441de6f9d6062e252d2eccb4a97'::bytea
            ,to_timestamp(1691479928)
            ,$$What doesn't?$$
            ,'text/html'
            ,'\x7ac84dba79c3c8c085e49f96a03a91ab24ad6c436b9c798d76e5bdf1a3b7de3d'::bytea
            ,'\xb76435c9a3d7db5e48cc086c59b0dac7d7212433854f8f0151bd847dcf0411f0c5c5f4468afe5e5e6acfb354a6251d52dec0a7ee6a99e7305525d1c855d59a0c'::bytea
            ,'\x3a1125503febce1f4bc474b41fd7b0c6ce8019570e4ca1a6c923daff43871c74'::bytea
            ,'wgt'
            ,'wgt@aggy.news'
        )
        ,(
            '\xdb67a59f1d0e48fb76a9e80e5dea10449d741cd5e3cc0ab55a3a9f3e210e7eed'::bytea
            ,to_timestamp(1691479928)
            ,$$Stop redditing!!!$$
            ,'text/html'
            ,'\xc6d9d817d53dee6c0ae00205e9f32f6373b23215ddd442a5dce193cce73f5925'::bytea
            ,'\x61f7af2598e3e091de1a94bc362b65d2e6bc1442bb8ae6c2fbecf7ea59c7c502eff44132d96e944ef36923194f29f0ff8a20a4c220e5886848d33737c377d406'::bytea
            ,'\x7b6e363d7bfd80fbe4af53b0c167fa44fce03fca1f9cc04d525b83f40e92c2ca'::bytea
            ,'ftw'
            ,'ftw@aggy.news'
        )
        ,(
            '\x2abd6980fedaf96871a82f4f71aa08a693925ae287cc2b44426859c4aa4b74f4'::bytea
            ,to_timestamp(1691479928)
            ,$$<a href="https://simple.news/p/atlantis-resurface">Atlantis resurfaces 20 miles off the coast of Hong Kong!</a>
<p>
This is an ongoing story. Please abstain from moralspeech or alterjecting.

Make sure to make use of pubkeys registered on the Bloodchain as per JURISPRUDENCE-COMMIT-9becb3c12. All unregistered pubkeys will be held liabale for any casualites and damage in case of flamewars.</p>$$
            ,'text/html'
            ,NULL
            ,'\x8de0773455a2a49708e9fa8223f04edd82a48fbdef10f299b001d2292fbc6f51e34b0e65cc497f942b4520597d6e2be78f27afc438dbeb3d57fcc7d16c3f7600'::bytea
            ,'\x7c5bade04be3bb0fb9bd33f5eec539863c0c82866e333e525311823ef44b8cf5'::bytea
            ,'sabrina'
            ,'sabrina@aggy.news'
        )
        ,(
            '\x8687716dd1632690fadde256551eb7733f58a34f0c7a61f3f9455da5bb6a4d0b'::bytea
            ,to_timestamp(1691479928)
            ,$$I'd like to know what the probablities of this being a psyop are considering international relations and the situation in the pacific?$$
            ,'text/html'
            ,'\x2abd6980fedaf96871a82f4f71aa08a693925ae287cc2b44426859c4aa4b74f4'::bytea
            ,'\x000e0f990c8df35ee04c692964f03a39f3d5cf30952632df125139f4f54309d38b3cf0f1a4d42f64f24170e01046199019bb03dfef65b2e8bdab67b95c263a09'::bytea
            ,'\x433d788d36ec57c3529e6c95a6b473244afd3abc8cef75129083e0e027b1472f'::bytea
            ,'archie'
            ,'archie@aggy.news'
        )
        ,(
            '\x21cc1cf52eaf06eb028cd7ada93f148e9f9b5d350a580da4f406f27706be498c'::bytea
            ,to_timestamp(1691479928)
            ,$$95% a psyop.$$
            ,'text/html'
            ,'\x8687716dd1632690fadde256551eb7733f58a34f0c7a61f3f9455da5bb6a4d0b'::bytea
            ,'\xe840016da30f70e4c49f50f2220e5cd20ce685e11e93fee2bd39eb9cc0b7b0f900c5a232dd4652598c24191959a208ce82f0c64206bf04e3b021bbc42f547b09'::bytea
            ,'\x7348c0e069deff565de5de523a1c4966ecf3318516da669f49ed76f5317b4830'::bytea
            ,'betty'
            ,'betty@aggy.news'
        )
        ,(
            '\xb311867587ecf0664c3b789abb9f3850be9fe373815446d4c670d9a5aafb5f4a'::bytea
            ,to_timestamp(1691479928)
            ,$$I was hoping for paragraphs.$$
            ,'text/html'
            ,'\x21cc1cf52eaf06eb028cd7ada93f148e9f9b5d350a580da4f406f27706be498c'::bytea
            ,'\x8a63489800a57b6974de6aa3cf79d539503f3eeb5846687c36a57d6031f991c26403f8e06396b2ae16cbf05c81bc072c82d7ee3ab6606a47985a683f6343b900'::bytea
            ,'\x433d788d36ec57c3529e6c95a6b473244afd3abc8cef75129083e0e027b1472f'::bytea
            ,'archie'
            ,'archie@aggy.news'
        )
        ,(
            '\x49997f33a1cf29863c65b6537788520c5cb0ead63816b07c14e913f14d2b5448'::bytea
            ,to_timestamp(1691479928)
            ,$$No one here knows enough for paragraphs.$$
            ,'text/html'
            ,'\xb311867587ecf0664c3b789abb9f3850be9fe373815446d4c670d9a5aafb5f4a'::bytea
            ,'\xff5d1c07817be94a3c506b7665a8195ac6cbdd5dac70c7dc58b60ed679dcdff4fa39d7faca2ee1b5b2412166b2d480e9c4bcaa0c4d79b62b9af2e7279e5a6708'::bytea
            ,'\x7348c0e069deff565de5de523a1c4966ecf3318516da669f49ed76f5317b4830'::bytea
            ,'betty'
            ,'betty@aggy.news'
        )
        ,(
            '\x78d50131dfe82290b81504c7a8c184266431882f7f0286a6b47ca9c590affff6'::bytea
            ,to_timestamp(1691479928)
            ,$$How do you do fellow terrestrials? We come in peace.$$
            ,'text/html'
            ,'\x2abd6980fedaf96871a82f4f71aa08a693925ae287cc2b44426859c4aa4b74f4'::bytea
            ,'\xd126fc81a53ae6ae138d4453e8ccf99be7c703fccddc84c9f89fd502e93c8ebe977f1a6a5bec4618ba49310a58675bd56815eeef3e85cec17e19eddd9290760b'::bytea
            ,'\xd46dcedda371eeb9d82fab2ca320a2654abcfae210ebf5046a44483e4bb53632'::bytea
            ,'veronica'
            ,'veronica@aggy.news'
        )
        ,(
            '\xbb0bef8da066687d2924de0d44e2b97a3cd34655fb0764f853869a7f40a8a7bd'::bytea
            ,to_timestamp(1691479928)
            ,$$How're you able to access this messageboard?$$
            ,'text/html'
            ,'\x78d50131dfe82290b81504c7a8c184266431882f7f0286a6b47ca9c590affff6'::bytea
            ,'\xc4265033590a2d12b4e2f67d2341d7ee097a0e5557cf5152a68061ac3375c4233e1573380ee9a1eed409b0d370ea6d91ae46c828bf1537c048070b17fa15a804'::bytea
            ,'\x7c5bade04be3bb0fb9bd33f5eec539863c0c82866e333e525311823ef44b8cf5'::bytea
            ,'sabrina'
            ,'sabrina@aggy.news'
        )
        ,(
            '\xdf7088d24bad0521fa9aba60c967756a69ae49d28f0f5bae68cb7323755ae2f5'::bytea
            ,to_timestamp(1691479928)
            ,$$Atlantis runs on a UNIX derivate.$$
            ,'text/html'
            ,'\xbb0bef8da066687d2924de0d44e2b97a3cd34655fb0764f853869a7f40a8a7bd'::bytea
            ,'\x1c0cb17667a4e5a3b413c0fe4f0e528bcc347d216234ef4ab33e9c02e5baad9dae167f281218d73793ff3df84ce07d63a57c1b2049635d056c87a45a9773350b'::bytea
            ,'\xd46dcedda371eeb9d82fab2ca320a2654abcfae210ebf5046a44483e4bb53632'::bytea
            ,'veronica'
            ,'veronica@aggy.news'
        )
        ,(
            '\xb724f8c17b7782bd72a471b37d90aea3c887bbcfe98b20d2de240e917cbb1043'::bytea
            ,to_timestamp(1691479928)
            ,$$<a href="https://aggy.news/p/a0c78830-d6c5-4133-af47-daac110aeb2c.txt">I suspect my wife of YDL membership</a>

<p>I first started to notice the signs a few weeks ago after I discovred somne inconsistency in my terminal history. Note: I'm currently employed an employee of Alphaborg at their Youtube division. Any advice is appreciated.</p>$$
            ,'text/html'
            ,NULL
            ,'\x0b6fe21bf0fb2116ac17ea062ad8381c002fb41bf08d16364d68ffce86168a0d7f09209adc849962a628410b7a25398bc2b98d98262b1fa279b651d6f8cf2605'::bytea
            ,'\x433d788d36ec57c3529e6c95a6b473244afd3abc8cef75129083e0e027b1472f'::bytea
            ,'archie'
            ,'archie@aggy.news'
        )
        ,(
            '\x29c7cf95f1eb5e3a9187d5b9a9b0e9375f744861cb1e226fe6a81c929c2d4d80'::bytea
            ,to_timestamp(1691479928)
            ,$$NOTE: Thread has been shut down due to unsanctioned flamewar.$$
            ,'text/html'
            ,'\xb724f8c17b7782bd72a471b37d90aea3c887bbcfe98b20d2de240e917cbb1043'::bytea
            ,'\x89a9173144e90ed9c78ba71cab0a5a40785e58b892ea3a03741d677b88eaa5a5a7e819572907c169e0195af0f9c3a2d0119cb4d67ebc0a09e33469cff3408002'::bytea
            ,'\x7348c0e069deff565de5de523a1c4966ecf3318516da669f49ed76f5317b4830'::bytea
            ,'betty'
            ,'betty@aggy.news'
        )
        ,(
            '\xf1fe48098ee8a9c3de6ad11d132f4bbfa5ddfe1e3ab0608b4a07aacadd4e69b9'::bytea
            ,to_timestamp(1691479928)
            ,$$<a href="nncp://857893/8471291/7583921748203.txt">Tokyo report shows record numbers of discarded limbs infesting underways</a>$$
            ,'text/html'
            ,NULL
            ,'\x5bac4db6d50f40e0b5bb90c252031aec177c727f410ac27e9957020b6f0beac1fc4d29ccd2e79d8475ef458ddd3dd12ec4c3398d5663c7f266d436398745740c'::bytea
            ,'\x433d788d36ec57c3529e6c95a6b473244afd3abc8cef75129083e0e027b1472f'::bytea
            ,'archie'
            ,'archie@aggy.news'
        )
        ,(
            '\x378d684f41f0896c67d3514d6ea6f4bc513a27f0220eb49256fd144dcc85d0e2'::bytea
            ,to_timestamp(1691479928)
            ,$$<a href="https://nil.null/89897898-rem-adware-danger">REM sleep adware considered dangerous</a>$$
            ,'text/html'
            ,NULL
            ,'\xe3e96c48c745892e08d5f708907cd64c21d81b667764ba3d13f49b09fe70e84c8fc7c533393e1af0f6cbf292a62c13bf9cc4c036f108b931db36dcb02ba19a0b'::bytea
            ,'\x7348c0e069deff565de5de523a1c4966ecf3318516da669f49ed76f5317b4830'::bytea
            ,'betty'
            ,'betty@aggy.news'
        )
        ,(
            '\x1285cb45d6495cf1ce6637179517a38758b2c0019dabf1b4492dc3e5d976cedd'::bytea
            ,to_timestamp(1691479928)
            ,$$<a href="https://arxiv.org/abs/31415.193">P=NP in 9 dimensions</a>$$
            ,'text/html'
            ,NULL
            ,'\x50b7888074b2223d16204d6420183a5cf6ae88ff434f41891e41a664efa42b45217b9d9797327ea8b2d2c3cb2783fcd688209aaa327c74344c30fc0d13970906'::bytea
            ,'\xd46dcedda371eeb9d82fab2ca320a2654abcfae210ebf5046a44483e4bb53632'::bytea
            ,'veronica'
            ,'veronica@aggy.news'
        )
;
    END;
$body$ LANGUAGE PLpgSQL;

-- you can bypass the DO section though
-- INSERT UPDATE STUFF
COMMIT;
-- ROLLBACK;
