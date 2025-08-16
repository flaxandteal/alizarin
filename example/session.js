const { ref } = Vue;

const name = ref("");
const slots = ref([]);

export default {
  name: "Session",
  props: {
    session: Object
  },
  async mounted() {
    this.session.name.then(n => { console.log('n'); name.value = n; });
    const slotsPromises = [1, 2, 3, 4, 5, 6].map(async (slot) => {
      const talk = this.session[`slot_${slot}`];
      const title = await talk.title;
      if (title) {
        const presenters = await talk.presenter;
        const presenterNames = await Promise.all(
          presenters.map(
            async (presenter) => (await presenter).name
          )
        );
        return {
          slot,
          title,
          presenters: presenterNames.join(', ')
        };
      }
      return slot;
    });
    slots.value = await Promise.all(slotsPromises);
  },
  setup() {
    return {
      name,
      slots
    };
  },
  template: `
    <h1>Session {{ name }}</h1>

    <ul>
      <li v-for="slot in slots">
        <span v-if="slot.title">
          {{ slot.slot }} : {{ slot.title }}
          <span v-if="slot.presenters">-- {{ slot.presenters }}</span>
        </span>
        <span v-else>(empty)</span>
      </li>
    </ul>
  `
}
