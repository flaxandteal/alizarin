import { AlizarinModel, graphManager } from 'alizarin'; // @alizcode-hide

async function run({print}: {print: ((...inp: any) => void)}) {
  try {
    class Person extends AlizarinModel<Person> {};

    const Persons = await graphManager.get(Person);
    print(Persons);
    const everyone: Person[] = await Persons.all({lazy: true});
    print(everyone);

    for (const person of everyone) {
      print(person.name, "->", person.friends[0].name);
    }
  } catch (e: any) {
    print(e)
  }
}
export default {run};
