-- Your SQL goes here
CREATE TABLE tasks (
    id int unsigned NOT NULL AUTO_INCREMENT,
    task_id varchar(100) NOT NULL,
    params text NOT NULL,
    result text NOT NULL,
    starts_at timestamp DEFAULT NULL,
    ends_at timestamp DEFAULT NULL,
    callback_url varchar(200) NOT NULL,
    created_at timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    PRIMARY KEY (`id`),
    UNIQUE KEY `tasks_unique_index_task_id` (`task_id`)
);
