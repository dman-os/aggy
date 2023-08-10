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
            '\x43203d798236edc55a20be5de2f9a401514b62e85ad0792fdffe4ca9b1b6a5a0'::bytea
            ,to_timestamp(1691479928)
            ,$$I wan't you to know, I wan't you to know that I'm awake.$$
            ,'text/html'
            ,NULL
            ,'\x2492541d9c12871570caeecb1aaec6d2197b5e058405393dc624a2299568055ec092fe91ea34a85b99cb45921fbab2999f5aa7f215956e4806f948b49a0d020d'::bytea
            ,'\xdc67469e70cbcab49a7840ab1b44d56c7963a7ca24c626ebb792fa7f514f37aa'::bytea
            ,'use1'
            ,'use1@aggy.news'
        )
        ,(
            '\xa52b926f893d3019cdbddbf4c553232c43035b4fe08eb26c31892e3b57b3dfc5'::bytea
            ,to_timestamp(1691479928)
            ,$$And I hope you're asleep.$$
            ,'text/html'
            ,'\x43203d798236edc55a20be5de2f9a401514b62e85ad0792fdffe4ca9b1b6a5a0'::bytea
            ,'\x8e5934a66506ec9742c0af2393fcc5158da433012284f071b8c4a47b721cec3db91057f59f3f609bc23eaabe97a55524aff30ff12eb18b1d3cf55eaf981e3601'::bytea
            ,'\x43366142dc9ce1022ce9ec9deb72b0242ed3e097c8d7531cad8bf982cf9edb7f'::bytea
            ,'fideroth'
            ,'fideroth@aggy.news'
        )
        ,(
            '\x733866a0fd613e6afbd4687d8c5a4116cd298403c31fecf88f84c5961fc53a23'::bytea
            ,to_timestamp(1691479928)
            ,$$*air guitars madly*$$
            ,'text/html'
            ,'\xa52b926f893d3019cdbddbf4c553232c43035b4fe08eb26c31892e3b57b3dfc5'::bytea
            ,'\xfae46b531cb4ef8e466baa4d14e0fa8eec249345387717f72b28789c559486ab1b17b88e3bf3f22adafc6654dc8fc606fcbcbfcb82bed4254ccf142ed2c05a05'::bytea
            ,'\xdc67469e70cbcab49a7840ab1b44d56c7963a7ca24c626ebb792fa7f514f37aa'::bytea
            ,'use1'
            ,'use1@aggy.news'
        )
        ,(
            '\x0ae08db70cf1ff1d6d56248e59c6ab3adb1f6e34e3fe3fee61f636cc2f4851e1'::bytea
            ,to_timestamp(1691479928)
            ,$$*sads doggly*$$
            ,'text/html'
            ,'\x733866a0fd613e6afbd4687d8c5a4116cd298403c31fecf88f84c5961fc53a23'::bytea
            ,'\x85b92cd7c5d5e79778cf7698f728b6fec0c34f94300c9c59abbe755f526df461d35819f19d3c3704192e9f2b0cb1dbfeb900710e32c2afcca9a2469d8a832d09'::bytea
            ,'\x43366142dc9ce1022ce9ec9deb72b0242ed3e097c8d7531cad8bf982cf9edb7f'::bytea
            ,'fideroth'
            ,'fideroth@aggy.news'
        )
        ,(
            '\xa67c768430edde87d5e85cff972cf4d5f2380852c08eb6b9ed78074e2be24839'::bytea
            ,to_timestamp(1691479928)
            ,$$What gives?$$
            ,'text/html'
            ,'\x43203d798236edc55a20be5de2f9a401514b62e85ad0792fdffe4ca9b1b6a5a0'::bytea
            ,'\x1f908f36e9d7270ff223eb683ff6fa154f22b3a804210e03487f5653c0a496d73f553ab432842e6db14f55fcdf33df6a2c68bb2743a81ef9a5824f923e89ab09'::bytea
            ,'\xc9e8342fe4acdf2263b6b28db67319cc3f39bca5f4164d72791889df373d4842'::bytea
            ,'the_i18n_man'
            ,'the_i18n_man@aggy.news'
        )
        ,(
            '\x866c79e7f970e997f676c1b7a1e8992b71bcf55e4830b677b3fdb4045055af02'::bytea
            ,to_timestamp(1691479928)
            ,$$What doesn't?$$
            ,'text/html'
            ,'\xa67c768430edde87d5e85cff972cf4d5f2380852c08eb6b9ed78074e2be24839'::bytea
            ,'\xb2629d183ee1f50948d72f8a2c4af231bcc8c892b04c5febe3a04f9f3e93027b91a9b9efb4fc6113a95d4c372e5fd4a6e2ff40ea8292e8a0c4c708bdaaf88a0f'::bytea
            ,'\xad9828dd4e3061f37508124b4843b42dfd109ac94366099e3e6df7666806d3f6'::bytea
            ,'wgt'
            ,'wgt@aggy.news'
        )
        ,(
            '\xa6089a9b605ea78f1b34604f20cec525c93a183223a9d3e3dbb2097cd7c2b1cb'::bytea
            ,to_timestamp(1691479928)
            ,$$Stop redditing!!!$$
            ,'text/html'
            ,'\x43203d798236edc55a20be5de2f9a401514b62e85ad0792fdffe4ca9b1b6a5a0'::bytea
            ,'\x741402cd5be00a0ac1bfef34ba6b2360cd3985b2962e82d1ba7aa5d851c490169a527a5421037b88dd640c7fd0d08d4c38cd33e2349fd1b61eeba8a44008de0f'::bytea
            ,'\x3581abc925c8a5f5cb9a6e5fe168ec231ac2a50c4b7292cfdef7a498c5882ec6'::bytea
            ,'ftw'
            ,'ftw@aggy.news'
        )
        ,(
            '\x65fdb61533b12c0be75352af20134ba2e7cc1d7ccba7c81a6eb3dad18ab7d0cd'::bytea
            ,to_timestamp(1691479928)
            ,$$<a href="https://simple.news/p/atlantis-resurface">Atlantis resurfaces 20 miles off the coast of Hong Kong!</a>$$
            ,'text/html'
            ,NULL
            ,'\xd0b46381a21ed65e5d0f86cd5fdc64336b3d8610f397f2d99f1b4a80298fa952e85a0e268011303910e21b4c19d18a7c275d3be818eac39611d3ca3a3fd2900d'::bytea
            ,'\x7c5bade04be3bb0fb9bd33f5eec539863c0c82866e333e525311823ef44b8cf5'::bytea
            ,'sabrina'
            ,'sabrina@aggy.news'
        )
        ,(
            '\xdc1b39fd3b63293a353328f69a20cf054db32c39fe183e5b16331ef78bc48227'::bytea
            ,to_timestamp(1691479928)
            ,$$I'd like to know what the probablities of this being a psyop are considering international relations and the situation in the pacific?$$
            ,'text/html'
            ,'\x65fdb61533b12c0be75352af20134ba2e7cc1d7ccba7c81a6eb3dad18ab7d0cd'::bytea
            ,'\xf721c55d35a5c847a709f6dad1c665c2442c9e199463763de3ec169f2469c4ffb3a9857d033b6d743bfccb2e2787ad4033bb36cee0611b5ba02d7ce10d94130f'::bytea
            ,'\x433d788d36ec57c3529e6c95a6b473244afd3abc8cef75129083e0e027b1472f'::bytea
            ,'archie'
            ,'archie@aggy.news'
        )
        ,(
            '\x085db90b4892a8109d37a4bf77f7a47a5f0a2de435dda9d329173dfc8fcfb572'::bytea
            ,to_timestamp(1691479928)
            ,$$95% a psyop.$$
            ,'text/html'
            ,'\xdc1b39fd3b63293a353328f69a20cf054db32c39fe183e5b16331ef78bc48227'::bytea
            ,'\xdac73deede8c2536d656205edaa50572759337825709b18e4c21b404d715593a03ffef1d1f1252c5be13048daa4a42d8d99d9061135a0b612c50e14b77d52505'::bytea
            ,'\x7348c0e069deff565de5de523a1c4966ecf3318516da669f49ed76f5317b4830'::bytea
            ,'betty'
            ,'betty@aggy.news'
        )
        ,(
            '\x597a236b9ea50c2ffbb26314e68c0716852c6a86ffa1a4a7f1cb5678ac23853b'::bytea
            ,to_timestamp(1691479928)
            ,$$I was hoping for paragraphs.$$
            ,'text/html'
            ,'\x085db90b4892a8109d37a4bf77f7a47a5f0a2de435dda9d329173dfc8fcfb572'::bytea
            ,'\xa65a6fccf70513a645f67ececa8d6c0aae12f00d7472bbf2bff23c632a88d277adb277a35822cdc1a2019d51ffa896198f97a055e706f6f35fd4bbd6d8bb3303'::bytea
            ,'\x433d788d36ec57c3529e6c95a6b473244afd3abc8cef75129083e0e027b1472f'::bytea
            ,'archie'
            ,'archie@aggy.news'
        )
        ,(
            '\x7fe3533f7aa61051b8fc12f3e7b8384f002180377a0a8d8caacb07743d7ec0f0'::bytea
            ,to_timestamp(1691479928)
            ,$$No one here knows enough for paragraphs.$$
            ,'text/html'
            ,'\x597a236b9ea50c2ffbb26314e68c0716852c6a86ffa1a4a7f1cb5678ac23853b'::bytea
            ,'\x896b96bdcbab79f04065efc37f0b55810aa02eaa16e5f6a49a570be57197827b014b830aada19d11069c24e162c93c85c983b75591597a56e51eab44397bd305'::bytea
            ,'\x7348c0e069deff565de5de523a1c4966ecf3318516da669f49ed76f5317b4830'::bytea
            ,'betty'
            ,'betty@aggy.news'
        )
        ,(
            '\x7bf7c9a55cf79d2d1263dbd775a224bba4c64d04897037a0eb9d1ae47c7707f6'::bytea
            ,to_timestamp(1691479928)
            ,$$How do you do fellow terrestrials? We come in peace.$$
            ,'text/html'
            ,'\x65fdb61533b12c0be75352af20134ba2e7cc1d7ccba7c81a6eb3dad18ab7d0cd'::bytea
            ,'\x47493656559c182f1a983aa8f54f674d93b155c58e82478fb7cf850865466931e4d313f8f897d2d45066c01955919433fbdf7215d5c021909b8dc87e9d9d0109'::bytea
            ,'\xd46dcedda371eeb9d82fab2ca320a2654abcfae210ebf5046a44483e4bb53632'::bytea
            ,'veronica'
            ,'veronica@aggy.news'
        )
        ,(
            '\x1c25b31ede76035fe0ed5178d90f4dca491671797d6af89bab0f66526af90e84'::bytea
            ,to_timestamp(1691479928)
            ,$$How're you able to access this messageboard?$$
            ,'text/html'
            ,'\x7bf7c9a55cf79d2d1263dbd775a224bba4c64d04897037a0eb9d1ae47c7707f6'::bytea
            ,'\xdcbd8c54614a7001af6bc809b5047751b586092e342b92e42347be2c8e49f56052ae1241d7250808ea12381827f0bff17e3cf8de39ae816d27426aabb5038700'::bytea
            ,'\x7c5bade04be3bb0fb9bd33f5eec539863c0c82866e333e525311823ef44b8cf5'::bytea
            ,'sabrina'
            ,'sabrina@aggy.news'
        )
        ,(
            '\x3d07d559f711233c9ba32207dda651c577954b7f10224125ab5d8cb11f66c125'::bytea
            ,to_timestamp(1691479928)
            ,$$Atlantis runs on a UNIX derivate.$$
            ,'text/html'
            ,'\x1c25b31ede76035fe0ed5178d90f4dca491671797d6af89bab0f66526af90e84'::bytea
            ,'\x681ddc61486f7e291656d0f76be2268e608a0c5e57dd2e62371a9371a34114bf045059ecfce45b91c320ced8ef06baedf124388f27b5f250879d1d04ac75d007'::bytea
            ,'\xd46dcedda371eeb9d82fab2ca320a2654abcfae210ebf5046a44483e4bb53632'::bytea
            ,'veronica'
            ,'veronica@aggy.news'
        )
        ,(
            '\x01e94b7091caea92f5d0f07a06efc088feaf848f44b6b65dfffa2df1357fcf7b'::bytea
            ,to_timestamp(1691479928)
            ,$$<a href="nncp://857893/8471291/7583921748203.txt">Tokyo report shows record numbers of discarded limbs infesting underways</a>$$
            ,'text/html'
            ,NULL
            ,'\xdd0020b113fe35ba2f9b6d6f7e8f26ef6feb50810bcc2ab1bc3735be56d99c7ab41d93b806266528d057897286885bd650370ef36cb2ad3b84cf9d902ca21b03'::bytea
            ,'\x433d788d36ec57c3529e6c95a6b473244afd3abc8cef75129083e0e027b1472f'::bytea
            ,'archie'
            ,'archie@aggy.news'
        )
        ,(
            '\x6a6ed2475a9d74e1a630b581617c975dd8c2a7fa1a53f3711fef9212bbb13504'::bytea
            ,to_timestamp(1691479928)
            ,$$<a href="https://nil.null/89897898-rem-adware-danger">REM sleep adware considered dangerous</a>$$
            ,'text/html'
            ,NULL
            ,'\x4cdcc809ae4881c6c8be320232d0cfb0182223ea38441f70e6c354f618a51dd3e4145b8fc1d3d0dde2f93856c8821a76f02f33554c6ebed59de77a0199319808'::bytea
            ,'\x7348c0e069deff565de5de523a1c4966ecf3318516da669f49ed76f5317b4830'::bytea
            ,'betty'
            ,'betty@aggy.news'
        )
        ,(
            '\xeb80db9a7dd7a0c4b1b02720993233cf12f6102f3f016c4b1b6340b7c3f2bee0'::bytea
            ,to_timestamp(1691479928)
            ,$$<a href="https://arxiv.org/abs/31415.193">P=NP in 9 dimensions</a>$$
            ,'text/html'
            ,NULL
            ,'\x8915f932a935e10e665b8f92bf205f67e1d4321a38f307113b9a7c9bbc45b7b1918384882ce099457fb29a8274a0ff8331046ac4f944f1d3f98ea5884350c606'::bytea
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
