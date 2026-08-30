using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_RequestActivateTargetedLocationPlayerPower
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.RequestActivateTargetedLocationPlayerPower); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.RequestActivateTargetedLocationPlayerPower)obj;
            //  Serialize AbilityId
            s.Write(value.AbilityId);
            //  Serialize Target
            s.Write(value.Target);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.RequestActivateTargetedLocationPlayerPower)) as Rts.CnC.Messages.Client.RequestActivateTargetedLocationPlayerPower;
            //  Deserialize AbilityId
            s.Read(out value.AbilityId);
            //  Deserialize Target
            s.Read(out value.Target);

            return value;
        }
        
    }
}
