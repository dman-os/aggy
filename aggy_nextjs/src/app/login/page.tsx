import { getCsrfToken } from '@/utils/index.server';
import { LoginForm } from './components';

export default function LoginPage({
  searchParams,
}: {
  searchParams: { [key: string]: string | string[] | undefined }
}) {
  const param = searchParams["redirectTo"];
  const redirectTo = Array.isArray(param) ? param[0] : param ?? "/";

  return (
    <>
      <h3>Login</h3>
      <LoginForm redirectTo={redirectTo} csrfToken={getCsrfToken()} />
    </>
  );
}
