import { getCsrfToken } from '@/utils/index.server';
import { RegisterForm } from './components';

export default function RegisterPage({
  searchParams,
}: {
  searchParams: { [key: string]: string | string[] | undefined }
}) {
  const param = searchParams["redirectTo"];
  const redirectTo = Array.isArray(param) ? param[0] : param ?? "/";

  return (
    <>
      <h3>Register</h3>
      <RegisterForm redirectTo={redirectTo} csrfToken={getCsrfToken()} />
    </>
  );
}
