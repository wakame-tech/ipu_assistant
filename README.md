# ipu-assistant
## 💬 cmds
- `!points ls`: ポイントを確認する
- `!points reset`: ポイントをリセットする
- `+<num>`: {{num}}分遅れる
- `!events ls`: 定期イベントを確認する
- `!events add <event> <cron>`: 定期イベント {{event}} を追加する
- `!events update <event> <cron>`: 定期イベント {{event}} を更新する
- `!events rm <event>`: 定期イベント {{event}} を削除する

## 🔥 deploy
### push `.env.prd`
```
heroku config:push -o -f .env.prd
```

### migration
```
cargo make migrate
```