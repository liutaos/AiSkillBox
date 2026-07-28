// Copyright (c) Mr_老鬼. All rights reserved.
// https://www.junjiestudio.top

use salvo::prelude::*;

use crate::config;

/// 获取当前配置
#[handler]
pub async fn get_config(_req: &mut Request, res: &mut Response) {
    let cfg = config::get();
    res.render(Json(serde_json::json!({
        "success": true,
        "data": {
            "listen_addr": cfg.listen_addr,
            "mcp_listen_addr": cfg.mcp_listen_addr,
            "db_path": cfg.db_path,
            "skills_dir": cfg.mcp.skills_dir,
            "log_level": cfg.log.level
        }
    })));
}
