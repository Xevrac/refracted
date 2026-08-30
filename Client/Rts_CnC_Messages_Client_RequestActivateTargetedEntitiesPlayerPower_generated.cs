using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_RequestActivateTargetedEntitiesPlayerPower
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.RequestActivateTargetedEntitiesPlayerPower); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.RequestActivateTargetedEntitiesPlayerPower)obj;
            //  Serialize AbilityId
            s.Write(value.AbilityId);
            //  Serialize Target
            s.Write(value.Target);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.RequestActivateTargetedEntitiesPlayerPower)) as Rts.CnC.Messages.Client.RequestActivateTargetedEntitiesPlayerPower;
            //  Deserialize AbilityId
            s.Read(out value.AbilityId);
            //  Deserialize Target
            s.Read(out value.Target);

            return value;
        }
        
    }
}
