-- Project alert kinds. Social / listing kinds stay enabled but never fire without live sources.

INSERT INTO alert_rules (kind, enabled, reason) VALUES
    ('lifecycle_transition', TRUE, 'Twin lifecycle phase change'),
    ('holder_concentration_change', TRUE, 'Top10 holder concentration delta'),
    ('risk_signal', TRUE, 'High or critical risk band'),
    ('community_growth_anomaly', TRUE, 'Unique accounts 24h — live social only'),
    ('social_anomaly', TRUE, 'Organicness / bot shift — live social only')
ON CONFLICT (kind) DO NOTHING;

INSERT INTO feature_flags (key, enabled, reason) VALUES
    ('plane.project_alerts', TRUE, 'Project desk alerts + token-scoped webhooks')
ON CONFLICT (key) DO NOTHING;
