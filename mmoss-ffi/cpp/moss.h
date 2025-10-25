#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef struct MobFactoryBuilderPtr {

} MobFactoryBuilderPtr;

typedef struct FactoryPtr {

} FactoryPtr;

typedef struct WorldPtr {

} WorldPtr;

void mmoss_init_log(uint8_t level, void (*callback)(uint8_t level, const char *message));

struct MobFactoryBuilderPtr *mmoss_client_factory_builder_new(void);

struct FactoryPtr *mmoss_client_factory_builder_build(struct MobFactoryBuilderPtr *builder);

struct WorldPtr *mmoss_client_world_new(const struct FactoryPtr *factory, const char *addr);

void mmoss_client_world_destroy(struct WorldPtr *world);

void mmoss_client_world_update(struct WorldPtr *world,
                               void (*on_spawn)(uint32_t entity, uint32_t mob_type),
                               void (*on_component_updated)(uint32_t entity, uint32_t id),
                               void (*on_component_added)(uint32_t entity,
                                                          uint32_t spawn_id,
                                                          uint32_t component_type,
                                                          uint32_t id));
