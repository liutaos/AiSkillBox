// Copyright (c) Mr_老鬼. All rights reserved.
// https://www.junjiestudio.top
// Derivative works must retain this copyright notice.

import axios from 'axios'

const api = axios.create({
  baseURL: '/api/admin',
  timeout: 10000,
  headers: {
    'Content-Type': 'application/json'
  }
})

// 响应拦截器
api.interceptors.response.use(
  response => response.data,
  error => {
    const message = error.response?.data?.message || error.message || '请求失败'
    return Promise.reject(new Error(message))
  }
)

// 技能管理
export const getSkills = (params) => api.get('/skills', { params })
export const getTrash = (params) => api.get('/trash', { params })
export const searchSkills = (query, tags) => api.post('/search', { query, tags })
export const deleteSkill = (skill_name) => api.post('/delete', { skill_name })
export const restoreSkill = (skill_name) => api.post('/restore', { skill_name })
export const permanentDelete = (skill_name) => api.post('/permanent_delete', { skill_name })
export const enableSkill = (skill_name) => api.post('/enable', { skill_name })
export const disableSkill = (skill_name) => api.post('/disable', { skill_name })

// 服务控制
export const startService = () => api.post('/start')
export const stopService = () => api.post('/stop')
export const restartService = () => api.post('/restart')
export const getStatus = () => api.get('/status')
export const refreshSkills = () => api.post('/refresh')

// 配置
export const getConfig = () => api.get('/config')

// 记忆管理
export const getMemories = (params) => api.get('/memories', { params })
export const searchMemories = (data) => api.post('/memories/search', data)
export const deleteMemory = (memory_id) => api.post('/memories/delete', { memory_id })
export const archiveMemories = (data) => api.post('/memories/archive', data)
export const getMemoryStats = () => api.get('/memories/stats')

// 问题库
export const getIssues = (params) => api.get('/issues', { params })
export const searchIssues = (data) => api.post('/issues/search', data)
export const deleteIssue = (issue_id) => api.post('/issues/delete', { issue_id })

export default api
