using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_RequestActivateLineTargetedPlayerPower
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.RequestActivateLineTargetedPlayerPower); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.RequestActivateLineTargetedPlayerPower)obj;
            //  Serialize AbilityId
            s.Write(value.AbilityId);
            //  Serialize Target
            s.Write(value.Target);
            //  Serialize NormalizedDirection
            s.Write(value.NormalizedDirection);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.RequestActivateLineTargetedPlayerPower)) as Rts.CnC.Messages.Client.RequestActivateLineTargetedPlayerPower;
            //  Deserialize AbilityId
            s.Read(out value.AbilityId);
            //  Deserialize Target
            s.Read(out value.Target);
            //  Deserialize NormalizedDirection
            s.Read(out value.NormalizedDirection);

            return value;
        }
        
    }
}
