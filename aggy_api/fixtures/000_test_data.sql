-- strings use single quotes
BEGIN;

DO $body$
    DECLARE
        -- use variables in order to be able to access properties using the dot operator
        le_user    auth.users;
    BEGIN
        INSERT INTO auth.users (
            id, username, email, pic_url
        ) VALUES (
            'add83cdf-2ab3-443f-84dd-476d7984cf75'::uuid,
            'sabrina',
            'hex.queen@teen.dj',
            'https://obj.teen.dj/d78xas'
        ) RETURNING * INTO le_user;
        INSERT INTO auth.credentials (
            user_id, pass_hash
        ) VALUES ( 
            le_user.id, 
            '$argon2i$v=19$m=4096,t=3,p=1$c29tZXNhbHQ$iWh06vD8Fy27wf9npn6FXWiCX4K6pW6Ue1Bnzz07Z8A'
        );
        INSERT INTO auth.sessions (
            token, user_id, expires_at
        ) VALUES (
            '9d827d5c-15bd-413c-9431-39ff96155d7b',
            le_user.id,
            CURRENT_TIMESTAMP + interval '7 days'
        );
        INSERT INTO web.sessions (
            id, user_id, expires_at, ip_addr, user_agent
        ) VALUES (
            '13e4cbdf-aa7c-43ca-990c-a8b468d44616'::uuid,
            le_user.id,
            CURRENT_TIMESTAMP + interval '7 days',
            '127.0.0.1'::inet,
            'ViolaWWW'
        );
    END;
$body$ LANGUAGE PLpgSQL;

DO $body$
    DECLARE
        le_user    auth.users;
    BEGIN
        INSERT INTO auth.users (
            id, username, email, pic_url
        ) VALUES (
            'ce4fe993-04d6-462e-af1d-d734fcc9639d'::uuid,
            'archie',
            'archie1941@poetry.ybn',
            'https://pictu.res/01'
        ) RETURNING * INTO le_user;
        INSERT INTO auth.credentials (
            user_id, pass_hash
        ) VALUES ( 
            le_user.id, 
            '$argon2i$v=19$m=4096,t=3,p=1$c29tZXNhbHQ$iWh06vD8Fy27wf9npn6FXWiCX4K6pW6Ue1Bnzz07Z8A'
        );
    END;
$body$ LANGUAGE PLpgSQL;

DO $body$
    DECLARE
        le_user    auth.users;
    BEGIN
        INSERT INTO auth.users (
            id, username, email, pic_url
        ) VALUES (
            'd437e73f-4610-462c-ab22-f94b76bba83a'::uuid,
            'betty',
            'pInXy@melt.shake',
            null
        ) RETURNING * INTO le_user;
        INSERT INTO auth.credentials (
            user_id, pass_hash
        ) VALUES ( 
            le_user.id, 
            '$argon2i$v=19$m=4096,t=3,p=1$c29tZXNhbHQ$iWh06vD8Fy27wf9npn6FXWiCX4K6pW6Ue1Bnzz07Z8A'
        );
    END;
$body$ LANGUAGE PLpgSQL;

DO $body$
    DECLARE
        le_user    auth.users;
    BEGIN
        INSERT INTO auth.users (
            id, username, email, pic_url
        ) VALUES (
            '68cf4d43-62d2-4202-8c50-c79a5f4dd1cc'::uuid,
            'veronica',
            'trekkiegirl@ln.pi',
            'ipns://goatsie'
        ) RETURNING * INTO le_user;
        INSERT INTO auth.credentials (
            user_id, pass_hash
        ) VALUES ( 
            le_user.id, 
            '$argon2i$v=19$m=4096,t=3,p=1$c29tZXNhbHQ$iWh06vD8Fy27wf9npn6FXWiCX4K6pW6Ue1Bnzz07Z8A'
        );
        INSERT INTO auth.sessions (
            token, user_id, expires_at
        ) VALUES (
            'ebd3b465-be17-4077-bc4a-add9f76b5028',
            le_user.id,
            CURRENT_TIMESTAMP + interval '7 days'
        );
        INSERT INTO web.sessions (
            id, user_id, expires_at, ip_addr, user_agent
        ) VALUES (
            '0a7f6a02-43a4-4738-b70c-0d66eb24459f'::uuid,
            le_user.id,
            CURRENT_TIMESTAMP + interval '7 days',
            '8.0.0.1'::inet,
            'ViolaWWW'
        );
    END;
$body$ LANGUAGE PLpgSQL;

-- you can bypass the DO section though
-- INSERT UPDATE STUFF
COMMIT;
