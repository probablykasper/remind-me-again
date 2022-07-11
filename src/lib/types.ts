export type Group = {
  title: string
  description: string
  enabled: boolean
  id: number
  cron: string
  next_date: number | null
}
