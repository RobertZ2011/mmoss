#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef struct FactoryBuilderPtr {

} FactoryBuilderPtr;

typedef struct FactoryPtr {

} FactoryPtr;

typedef struct WorldPtr {

} WorldPtr;

struct FactoryBuilderPtr *mmoss_client_factory_builder_new(void);

struct FactoryPtr *mmoss_client_factory_builder_build(struct FactoryBuilderPtr *builder);

struct WorldPtr *client_world_new(const struct FactoryPtr *factory, const char *addr);

void client_world_destroy(struct WorldPtr *world);

void client_world_update(struct WorldPtr *world);
