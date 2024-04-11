// place files you want to import through the `$lib` alias in this folder.

export const host = "http://localhost:8080";

export async function checkUser(name: string): boolean {
  let res = await fetch(host + '/checkuser/' + name);
  return res.text === 'found';
}

export async function createUser(name: string, password: string): boolean {
  let res = await fetch(host + '/createuser/' + name + '/' + password, {method: 'POST'});
  return res.status === 200;
}

export async function loginUser(name: string, password: string): boolean {
  let res = await fetch(host + '/login/' + name + '/' + password, {method: 'POST'});
  return res.status === 200;
}