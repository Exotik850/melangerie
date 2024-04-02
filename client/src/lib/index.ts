// place files you want to import through the `$lib` alias in this folder.

export async function checkUser(name: string): boolean {
  let res = await fetch('/checkuser/' + name);
  return res.text === 'found';
}

export async function createUser(name: string): boolean {
  let res = await fetch('/createuser/' + name, {method: 'POST'});
  return res.status === 200;
}