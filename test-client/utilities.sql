-- 查询任务的执行时间
SELECT
	JSON_EXTRACT(params, '$.prompt') as input, task_id,
    starts_at, ends_at, timestampdiff(SECOND, starts_at, ends_at) as dur_1,
    created_at, timestampdiff(SECOND, created_at, ends_at) as dur_2
FROM `yum-backend`.tasks
ORDER BY id DESC
LIMIT 10
