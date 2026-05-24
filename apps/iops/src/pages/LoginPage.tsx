import { Button } from "@/components/ui/button";
import {
  Field,
  FieldError,
  FieldGroup,
  FieldLabel,
} from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import { invoke } from "@tauri-apps/api/core";
import { Link } from "react-router";
import * as z from "zod";
import { Controller, useForm } from "react-hook-form";
import { standardSchemaResolver } from "@hookform/resolvers/standard-schema";
import appIcon from "@/assets/app-icon.png";

const loginSchema = z.object({
  email: z.email("Email không hợp lệ"),
  password: z.string().min(8, "Email và mật khẩu không hợp lệ."),
});

type LoginValues = z.infer<typeof loginSchema>;

const LoginPage = () => {
  const form = useForm<LoginValues>({
    resolver: standardSchemaResolver(loginSchema),
    defaultValues: {
      email: "gaconght@gmail.com",
      password: "@Abc123123",
    },
  });

  async function onSubmit(data: LoginValues) {
    console.log(data);
    await invoke("open_main_window");
  }

  return (
    <main className="bg-background p-4 mx-auto max-w-sm flex items-center justify-center">
      <form
        className="flex flex-col gap-6 w-full"
        onSubmit={form.handleSubmit(onSubmit)}
      >
        <FieldGroup>
          <div className="flex flex-col items-center gap-1 text-center">
            <img src={appIcon} alt="I.C.H" className="mb-2 h-20 w-auto" />
          </div>
          <div className="flex flex-col items-center gap-1 text-center">
            <h1 className="text-2xl font-bold">Login to your account</h1>
            <p className="text-sm text-balance text-muted-foreground">
              Enter your email below to login to your account
            </p>
          </div>

          <Controller
            name="email"
            control={form.control}
            render={({ field, fieldState }) => (
              <Field data-invalid={fieldState.invalid}>
                <FieldLabel htmlFor={field.name}>Email</FieldLabel>
                <Input
                  {...field}
                  id={field.name}
                  type="email"
                  aria-invalid={fieldState.invalid}
                  placeholder="m@example.com"
                  required
                  autoComplete="off"
                />
              </Field>
            )}
          />

          <Controller
            name="password"
            control={form.control}
            render={({ field, fieldState }) => (
              <Field>
                <div className="flex items-center">
                  <FieldLabel htmlFor={field.name}>Password</FieldLabel>
                  <Link
                    to="/forgot-password"
                    className="ml-auto text-sm underline-offset-4 hover:underline"
                  >
                    Forgot your password?
                  </Link>
                </div>
                <Input
                  {...field}
                  id={field.name}
                  type="password"
                  placeholder="********"
                  aria-invalid={fieldState.invalid}
                  required
                  autoComplete="off"
                />
                {fieldState.invalid && (
                  <FieldError errors={[fieldState.error]} />
                )}
              </Field>
            )}
          />

          <Field>
            <Button type="submit">Login</Button>
          </Field>
        </FieldGroup>
      </form>
    </main>
  );
};

export default LoginPage;
