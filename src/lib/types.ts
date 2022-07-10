export type Group = {
  title: string
  description: string
  enabled: boolean
  id: number
  repeat: 'never' | 'daily'
  nextDate: Date | null
}
