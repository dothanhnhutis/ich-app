import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_protected/roles')({
  component: RouteComponent,
})

function RouteComponent() {
  return <div>Hello "/roles"!</div>
}
