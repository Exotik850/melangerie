// place files you want to import through the `$lib` alias in this folder.

export const host = "http://localhost:8080";

export async function checkUser(name: string): boolean {
  let res = await fetch(host + '/checkuser/' + name);
  return res.text === 'found';
}

export async function createUser(name: string, password: string): string | null {
  let res = await fetch(host + '/createuser', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/x-www-form-urlencoded',
    },
    body: 'name=' + name + '&password=' + password,
  });
  if (res.status === 200) {
    return await res.text();
  }
  return null;
}

export async function loginUser(name: string, password: string): string | null {
  let res = await fetch(host + '/login', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/x-www-form-urlencoded',
    },
    body: 'name=' + name + '&password=' + password,
  });
  if (res.status === 200) {
    return await res.text();
  }
  return null;
}