using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_TechTreeUnitAbilityActivated
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.TechTreeUnitAbilityActivated); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.TechTreeUnitAbilityActivated)obj;
            //  Serialize DependencyType
            s.Write(value.DependencyType);
            //  Serialize PlayerId
            s.Write(value.PlayerId);
            //  Serialize AbilityId
            s.Write(value.AbilityId);
            //  Serialize IsActivated
            s.Write(value.IsActivated);
            //  Serialize Dependency
            s.Write(value.Dependency);
            //  Serialize InstanceId
            s.Write(value.InstanceId);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.TechTreeUnitAbilityActivated)) as Rts.CnC.Messages.Client.TechTreeUnitAbilityActivated;
            //  Deserialize DependencyType
            s.Read(out value.DependencyType);
            //  Deserialize PlayerId
            s.Read(out value.PlayerId);
            //  Deserialize AbilityId
            s.Read(out value.AbilityId);
            //  Deserialize IsActivated
            s.Read(out value.IsActivated);
            //  Deserialize Dependency
            s.Read(out value.Dependency);
            //  Deserialize InstanceId
            s.Read(out value.InstanceId);

            return value;
        }
        
    }
}
