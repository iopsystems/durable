
extern void durable_ctor(void);

__attribute__((constructor, used)) void durable_ctor_wrapper(void) {
    durable_ctor();
}
