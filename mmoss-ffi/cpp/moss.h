#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef struct MobFactoryBuilderPtr {

} MobFactoryBuilderPtr;

typedef struct MobFactoryPtr {

} MobFactoryPtr;

typedef struct ComponentFactoryBuilderPtr {

} ComponentFactoryBuilderPtr;

typedef struct ComponentFactoryPtr {

} ComponentFactoryPtr;

typedef struct WorldPtr {

} WorldPtr;

/**
 * FFI-compatible 3D vector
 */
typedef struct Vec3 {
  float x;
  float y;
  float z;
} Vec3;

/**
 * FFI-compatible quaternion
 */
typedef struct Quat {
  float x;
  float y;
  float z;
  float w;
} Quat;

void mmoss_init_log(uint8_t level, void (*callback)(uint8_t level, const char *message));

struct MobFactoryBuilderPtr *mmoss_client_factory_builder_new(void);

struct MobFactoryPtr *mmoss_client_factory_builder_build(struct MobFactoryBuilderPtr *builder);

struct ComponentFactoryBuilderPtr *mmoss_client_component_factory_builder_new(void);

struct ComponentFactoryPtr *mmoss_client_component_factory_builder_build(struct ComponentFactoryBuilderPtr *builder);

struct WorldPtr *mmoss_client_world_new(const struct MobFactoryPtr *mob_factory,
                                        const struct ComponentFactoryPtr *component_factory,
                                        const char *addr);

void mmoss_client_world_destroy(struct WorldPtr *world);

void mmoss_client_world_update(struct WorldPtr *world,
                               void (*on_spawn)(uint64_t entity, uint32_t mob_type),
                               void (*on_component_updated)(uint64_t entity, uint32_t id),
                               void (*on_component_added)(uint64_t entity,
                                                          uint32_t spawn_id,
                                                          uint32_t component_type,
                                                          uint32_t id));

void mmoss_dynamic_actor_proxy_get_tranform(struct WorldPtr *world,
                                            uint64_t entity,
                                            struct Vec3 *out_translation,
                                            struct Quat *out_rotation);
