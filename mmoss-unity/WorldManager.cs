using UnityEngine;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System;

public class WorldManager : MonoBehaviour
{
    protected MmossFfi.WorldPtr world;
    protected Dictionary<ulong, GameObject> spawnedEntities = new Dictionary<ulong, GameObject>();
    protected Dictionary<uint, DynamicActorProxy> replicatedComponents = new Dictionary<uint, DynamicActorProxy>();

    static void LogCallback(byte level, string message)
    {
        Debug.Log($"[MMOSS][Level {level}] {message}");
    }

    void OnSpawnCallback(ulong entity, uint mobType)
    {
        Debug.Log($"[MMOSS] Spawned entity {entity} of mob type {mobType}");
        GameObject sphere = GameObject.CreatePrimitive(PrimitiveType.Sphere);
        sphere.transform.localScale = new Vector3(5f, 5f, 5f);
        spawnedEntities[entity] = sphere;
        Debug.Log($"Number of spawned entities: {spawnedEntities.Count}");
    }

    void OnComponentUpdatedCallback(ulong entity, uint id)
    {
        DynamicActorProxy proxy;
        if (replicatedComponents.TryGetValue(id, out proxy))
        {
            proxy.UpdateTransform(this.world);
        }
    }

    void OnComponentAddedCallback(ulong entity, uint spawnId, uint componentType, uint id)
    {
        Debug.Log($"[MMOSS] Added component {componentType} with id {id} on entity {entity} (spawn id {spawnId})");
        if (componentType == 7)
        {
            GameObject go;
            Debug.Log($"Trying to add DynamicActorProxy for entity {entity}");
            if (spawnedEntities.TryGetValue(entity, out go))
            {
                DynamicActorProxy proxy = go.AddComponent<DynamicActorProxy>();
                proxy.EntityId = entity;
                replicatedComponents[id] = proxy;
            }
        }
    }

    // Start is called once before the first execution of Update after the MonoBehaviour is created
    void Start()
    {
        MmossFfi.mmoss_init_log(4, LogCallback);

        MmossFfi.MobFactoryBuilderPtr builder = MmossFfi.mmoss_client_factory_builder_new();
        MmossExamplesFfi.mmoss_examples_lib_register_square_client(builder);

        MmossFfi.ComponentFactoryBuilderPtr componentBuilder = MmossFfi.mmoss_client_component_factory_builder_new();
        MmossExamplesFfi.moss_examples_lib_register_factory_components(componentBuilder);

        MmossFfi.MobFactoryPtr mob_factory = MmossFfi.mmoss_client_factory_builder_build(builder);
        MmossFfi.ComponentFactoryPtr component_factory = MmossFfi.mmoss_client_component_factory_builder_build(componentBuilder);

        Debug.Log("World manager start up");
        this.world = MmossFfi.mmoss_client_world_new(mob_factory, component_factory,"127.0.0.1:8080");
        MmossExamplesFfi.mmoss_examples_lib_world_register_components(this.world);
    }

    // Update is called once per frame
    void Update()
    {
        MmossFfi.mmoss_client_world_update(
            this.world,
            OnSpawnCallback,
            OnComponentUpdatedCallback,
            OnComponentAddedCallback);
    }

    void OnDestroy()
    {
        MmossFfi.mmoss_client_world_destroy(this.world);
    }
}
